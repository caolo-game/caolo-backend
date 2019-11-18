use crate::{scalar::Scalar, traits::ByteEncodeProperties, CompiledProgram, Instruction};
use arrayvec::{ArrayString, ArrayVec};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Inputs = HashMap<NodeId, ArrayVec<[NodeId; 16]>>;
pub type Nodes = BTreeMap<NodeId, AstNode>;

const INPUT_STR_LEN: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN]>;

impl ByteEncodeProperties for InputString {
    const BYTELEN: usize = INPUT_STR_LEN;

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
}

impl Default for AstNode {
    fn default() -> Self {
        Self {
            instruction: Instruction::Pass,
            string: None,
            scalar: None,
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
        ScalarLabel | ScalarInt | ScalarFloat | ScalarPtr | Pass | CopyLast => Some(0),
        StringLiteral | Exit | Call | ScalarArray => None,
    }
}

/// Single unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    pub nodes: Nodes,
    pub inputs: Inputs,
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
        let leafs: Vec<NodeId> = compiler
            .unit
            .nodes
            .iter()
            .map(|(k, _)| k)
            .filter(|k| {
                for it in compiler.unit.inputs.values() {
                    for n in it {
                        if n == *k {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        for nodeid in leafs.into_iter() {
            compiler.process_node(nodeid)?;
            compiler.program.bytecode.push(Instruction::Exit as u8);
        }

        Ok(compiler.program)
    }

    fn process_node(&mut self, nodeid: NodeId) -> Result<(), String> {
        use Instruction::*;

        Compiler::validate_node(nodeid, &mut self.unit)?;
        self.program
            .labels
            .insert(nodeid, self.program.bytecode.len());

        if let Some(inputs) = self.unit.inputs.get(&nodeid) {
            for nodeid in inputs.clone().into_iter() {
                self.process_node(nodeid)?;
            }
        }
        let instruction = {
            let node = &self
                .unit
                .nodes
                .get(&nodeid)
                .ok_or_else(|| format!("node [{}] not found in `nodes`", nodeid))?;
            node.instruction
        };
        if input_per_instruction(instruction)
            .map(usize::from)
            .map(|n| {
                n != 0
                    && self
                        .unit
                        .inputs
                        .get(&nodeid)
                        .map(|x| x.len() != n)
                        .unwrap_or(true)
            })
            .unwrap_or(false)
        {
            return Err(format!(
                "{:?} received invalid input. Expected: {:?} Actual: {:?}",
                instruction,
                input_per_instruction(instruction),
                self.unit.inputs.get(&nodeid)
            ));
        }

        match instruction {
            Exit | Pass | CopyLast | Branch | Add | Sub | Mul | Div => {
                self.push_node(nodeid);
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
                        if self
                            .unit
                            .inputs
                            .get(&nodeid)
                            .map(|x| x.len() != v as usize)
                            .unwrap_or(v != 0)
                        {
                            return Err("Array literal got invalid inputs".to_owned());
                        }

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
            ScalarLabel | ScalarPtr | ScalarFloat | ScalarInt => {
                self.push_node(nodeid);
                let value = self.unit.nodes[&nodeid]
                    .scalar
                    .ok_or_else(|| format!("node [{}] missing `scalar`", nodeid))?;
                match (instruction, value) {
                    (Instruction::ScalarInt, Scalar::Integer(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::ScalarFloat, Scalar::Floating(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::ScalarPtr, Scalar::Pointer(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::ScalarLabel, Scalar::Integer(v)) => {
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

    pub fn validate_node(node: NodeId, cu: &CompilationUnit) -> Result<(), String> {
        if let Some(n) = input_per_instruction(
            cu.nodes
                .get(&node)
                .ok_or_else(|| format!("node [{}] not found in `nodes`", node))?
                .instruction,
        ) {
            let n_inputs = cu.inputs.get(&node).map(|x| x.len()).unwrap_or(0)
                + cu.nodes[&node].string.map(|_| 1).unwrap_or(0);
            if n_inputs != n as usize {
                return Err(format!(
                    "Invalid number of inputs, expected {} got {}",
                    n, n_inputs
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_compiling_simple_program() {
        let nodes: Nodes = [
            (
                0,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(42.0)),
                    ..Default::default()
                },
            ),
            (
                1,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(512.0)),
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
        let inputs: Inputs = [(2, [0, 1].into_iter().cloned().collect())]
            .into_iter()
            .cloned()
            .collect();
        let program = CompilationUnit { nodes, inputs };

        let program = Compiler::compile(program).unwrap();

        println!("{:?}", program);

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

    /// Add val1 and val2 if true else subtract
    fn simple_branch_test(val1: f32, val2: f32, cond: i32, expected: f32) {
        let nodes: Nodes = [
            (
                10,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(val1)),
                    string: None,
                },
            ),
            (
                1,
                AstNode {
                    instruction: Instruction::ScalarFloat,
                    scalar: Some(Scalar::Floating(val2)),
                    string: None,
                },
            ),
            (
                2,
                AstNode {
                    instruction: Instruction::Add,
                    ..Default::default()
                },
            ),
            (
                5,
                AstNode {
                    instruction: Instruction::Sub,
                    ..Default::default()
                },
            ),
            (
                6,
                AstNode {
                    instruction: Instruction::ScalarInt, // Cond
                    scalar: Some(Scalar::Integer(cond)),
                    string: None,
                },
            ),
            (
                7,
                AstNode {
                    instruction: Instruction::ScalarLabel, // True
                    scalar: Some(Scalar::Integer(2)),
                    string: None,
                },
            ),
            (
                8,
                AstNode {
                    instruction: Instruction::ScalarLabel, // False
                    scalar: Some(Scalar::Integer(5)),
                    string: None,
                },
            ),
            (
                0,
                AstNode {
                    instruction: Instruction::Branch,
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .cloned()
        .collect();
        let inputs: Inputs = [
            (2, [10, 1].into_iter().cloned().collect()),
            (5, [10, 1].into_iter().cloned().collect()),
            (0, [6, 7, 8].into_iter().cloned().collect()),
        ]
        .into_iter()
        .cloned()
        .collect();
        let program = CompilationUnit { nodes, inputs };

        let program = Compiler::compile(program).expect("compile");

        let mut vm = VM::new(());
        if let Err(e) = vm.run(&program) {
            panic!("Err:{:?}\n{:?}", e, vm);
        }

        assert_eq!(vm.stack().len(), 1);

        let value = vm.stack().last().unwrap();
        match value {
            Scalar::Floating(i) => assert_eq!(*i, expected),
            _ => panic!("Invalid value in the stack"),
        }
    }

    #[test]
    fn test_branching_true() {
        simple_logger::init().unwrap_or(());
        simple_branch_test(42.0, 512.0, 1, 42.0 + 512.0);
    }

    #[test]
    fn test_branching_false() {
        simple_logger::init().unwrap_or(());
        simple_branch_test(42.0, 512.0, 0, 42.0 - 512.0);
    }
}
