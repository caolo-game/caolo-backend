use crate::{
    scalar::Scalar, traits::ByteEncodeProperties, CompiledProgram, InputString, Instruction,
    INPUT_STR_LEN,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Inputs = Vec<NodeId>;
pub type Nodes = BTreeMap<NodeId, AstNode>;

impl ByteEncodeProperties for InputString {
    const BYTELEN: usize = INPUT_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }

    fn encode(self) -> Vec<u8> {
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.chars().map(|c| c as u8));
        rr
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let len = i32::decode(bytes)?;
        let mut res = Self::new();
        for byte in bytes
            .iter()
            .skip(i32::BYTELEN)
            .take(len as usize)
            .map(|c| *c as char)
        {
            res.push(byte);
        }
        Some(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub instruction: Instruction,
    pub string: Option<InputString>,
    pub scalar: Option<Scalar>,
    pub children: Option<Inputs>,
    pub nodeid: Option<NodeId>,
}

impl Default for AstNode {
    fn default() -> Self {
        Self {
            instruction: Instruction::Pass,
            string: None,
            scalar: None,
            children: None,
            nodeid: None,
        }
    }
}

/// The accepted number of inputs of an instruction
/// None if unspecified
pub fn input_per_instruction(inst: Instruction) -> Option<u8> {
    use Instruction::*;
    match inst {
        Add | Sub | Mul | Div => Some(2),
        Branch => Some(3),
        WriteReg | Jump => Some(1),
        Start | ReadReg | ScalarLabel | ScalarInt | ScalarFloat | CopyLast => Some(0),
        Pass | StringLiteral | Exit | Call | ScalarArray => None,
    }
}

/// Single unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    pub nodes: Nodes,
}

pub struct Compiler {
    unit: CompilationUnit,
    program: CompiledProgram,
}

impl Compiler {
    pub fn compile(unit: CompilationUnit) -> Result<CompiledProgram, String> {
        if unit.nodes.is_empty() {
            return Err("Can not compile program with no entry point!".to_owned());
        }
        let mut compiler = Compiler {
            unit,
            program: CompiledProgram::default(),
        };
        let start = compiler
            .unit
            .nodes
            .iter()
            .find(|(_, v)| match v.instruction {
                Instruction::Start => true,
                _ => false,
            })
            .ok_or_else(|| "No start node has been found")?;

        let mut todo = VecDeque::with_capacity(compiler.unit.nodes.len());
        todo.push_back(*start.0);

        while !todo.is_empty() {
            let current = todo.pop_front().unwrap();
            compiler.process_node(current)?;
            if let Some(ref nodes) = compiler.unit.nodes[&current].children {
                for node in nodes.iter().cloned() {
                    todo.push_back(node);
                }
            } else {
                compiler.program.bytecode.push(Instruction::Exit as u8);
            }
        }

        Ok(compiler.program)
    }

    fn process_node(&mut self, nodeid: NodeId) -> Result<(), String> {
        use Instruction::*;

        let node = self
            .unit
            .nodes
            .get(&nodeid)
            .ok_or_else(|| format!("node [{}] not found in `nodes`", nodeid))?
            .clone();

        let fromlabel = self.program.bytecode.len();
        self.program
            .labels
            .insert(nodeid, [fromlabel, self.program.bytecode.len()]);

        let instruction = node.instruction;

        match instruction {
            Start | Exit | Pass | CopyLast | Branch | Add | Sub | Mul | Div => {
                self.push_node(nodeid);
            }
            Jump => {
                self.push_node(nodeid);
                let label = node
                    .nodeid
                    .ok_or_else(|| "Jump instruction requires a NodeId input")?;
                self.program.bytecode.append(&mut label.encode());
            }
            StringLiteral | Call => {
                self.push_node(nodeid);
                self.program.bytecode.append(
                    &mut self.unit.nodes[&nodeid]
                        .string
                        .ok_or_else(|| format!("node [{}] missing `string`", nodeid))?
                        .encode(),
                );
            }
            ScalarArray => {
                self.push_node(nodeid);
                match self.unit.nodes[&nodeid]
                    .scalar
                    .ok_or_else(|| format!("node [{}] missing `scalar`", nodeid))?
                {
                    Scalar::Integer(v) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    _ => {
                        return Err(format!(
                            "ScalarArray got invalid value {:?}",
                            self.unit.nodes[&nodeid].scalar
                        ))
                    }
                }
            }
            ReadReg | WriteReg | ScalarLabel | ScalarFloat | ScalarInt => {
                self.push_node(nodeid);
                let value = self.unit.nodes[&nodeid]
                    .scalar
                    .ok_or_else(|| format!("node [{}] missing `scalar`", nodeid))?;
                match (instruction, value) {
                    (Instruction::WriteReg, Scalar::Integer(v))
                    | (Instruction::ScalarLabel, Scalar::Integer(v))
                    | (Instruction::ReadReg, Scalar::Integer(v))
                    | (Instruction::ScalarInt, Scalar::Integer(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::ScalarFloat, Scalar::Floating(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    _ => {
                        return Err(format!(
                            "Scalar {:?} got invalid value {:?}",
                            instruction, value
                        ))
                    }
                }
            }
        }
        Ok(())
    }

    fn push_node(&mut self, nodeid: NodeId) {
        if let Some(node) = &self.unit.nodes.get(&nodeid) {
            self.program.bytecode.push(node.instruction as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_compiling_simple_program() {
        simple_logger::init().unwrap_or(());
        let nodes: Nodes = [
            (
                999,
                AstNode {
                    instruction: Instruction::Start,
                    children: Some(vec![0]),
                    ..Default::default()
                },
            ),
            (
                0,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(42.0)),
                    children: Some(vec![1]),
                    ..Default::default()
                },
            ),
            (
                1,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(512.0)),
                    children: Some(vec![2]),
                    ..Default::default()
                },
            ),
            (
                2,
                AstNode {
                    instruction: Instruction::Add,
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .cloned()
        .collect();

        let program = CompilationUnit { nodes };
        let program = Compiler::compile(program).unwrap();

        log::warn!("{:?}", program);

        // Compilation was successful

        let mut vm = VM::new(());
        vm.run(&program).unwrap();

        assert_eq!(vm.stack().len(), 1, "{:?}", vm.stack());

        let value = vm.stack().last().unwrap();
        match value {
            Scalar::Floating(i) => assert_eq!(*i, 42.0 + 512.0),
            _ => panic!("Invalid value in the stack"),
        }
    }
}
