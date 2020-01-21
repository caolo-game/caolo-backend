//! Compiles Graphs with vertices of `AstNode` into _caol-lang_ bytecode.
//! Programs must start with a `Start` instruction.
//!
mod astnode;
use crate::{
    traits::ByteEncodeProperties, CompiledProgram, InputString, Instruction, INPUT_STR_LEN,
};
pub use astnode::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
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
            return Err("Program is empty!".to_owned());
        }
        let mut compiler = Compiler {
            unit,
            program: CompiledProgram::default(),
        };
        let start = compiler
            .unit
            .nodes
            .iter()
            .find(|(_, v)| match v.node.instruction() {
                Instruction::Start => true,
                _ => false,
            })
            .ok_or_else(|| "No start node has been found")?;

        let mut nodes = compiler
            .unit
            .nodes
            .iter()
            .map(|(k, _)| *k)
            .collect::<BTreeSet<_>>();
        let mut todo = VecDeque::with_capacity(compiler.unit.nodes.len());
        todo.push_back(*start.0);

        loop {
            while !todo.is_empty() {
                let current = todo.pop_front().unwrap();
                nodes.remove(&current);
                compiler.process_node(current)?;
                match compiler.unit.nodes[&current].child.as_ref() {
                    None => compiler.program.bytecode.push(Instruction::Exit as u8),
                    Some(node) => {
                        todo.push_back(*node);
                    }
                }
            }
            match nodes.iter().next() {
                Some(node) => todo.push_back(*node),
                None => break,
            }
        }

        Ok(compiler.program)
    }

    fn process_node(&mut self, nodeid: NodeId) -> Result<(), String> {
        use InstructionNode::*;

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

        let instruction = node.node;

        match instruction {
            Equals | Less | LessOrEq | NotEquals | Exit | Start | Pass | CopyLast | Add | Sub
            | Mul | Div => {
                self.push_node(nodeid);
            }
            JumpIfTrue(j) | Jump(j) => {
                self.push_node(nodeid);
                let label = j.nodeid;
                self.program.bytecode.append(&mut label.encode());
            }
            StringLiteral(c) => {
                self.push_node(nodeid);
                self.program.bytecode.append(&mut c.value.encode());
            }
            Call(c) => {
                self.push_node(nodeid);
                self.program.bytecode.append(&mut c.function.encode());
            }
            ScalarArray(n) => {
                self.push_node(nodeid);
                self.program.bytecode.append(&mut n.value.encode());
            }
            ReadReg(r) | WriteReg(r) => {
                self.push_node(nodeid);
                let value = r.register;
                self.program.bytecode.append(&mut value.encode());
            }
            ScalarLabel(s) | ScalarInt(s) => {
                self.push_node(nodeid);
                self.program.bytecode.append(&mut s.value.encode());
            }
            ScalarFloat(s) => {
                self.push_node(nodeid);
                self.program.bytecode.append(&mut s.value.encode());
            }
        }
        Ok(())
    }

    fn push_node(&mut self, nodeid: NodeId) {
        if let Some(node) = &self.unit.nodes.get(&nodeid) {
            self.program.bytecode.push(node.node.instruction() as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn compiling_simple_program() {
        simple_logger::init().unwrap_or(());
        let nodes: Nodes = [
            (
                999,
                AstNode {
                    node: InstructionNode::Start,
                    children: Some(vec![0]),
                },
            ),
            (
                0,
                AstNode {
                    node: InstructionNode::ScalarFloat(FloatNode { value: 42.0 }),
                    children: Some(vec![1]),
                },
            ),
            (
                1,
                AstNode {
                    node: InstructionNode::ScalarFloat(FloatNode { value: 512.0 }),
                    children: Some(vec![2]),
                },
            ),
            (
                2,
                AstNode {
                    node: InstructionNode::Add,
                    children: None,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let program = CompilationUnit { nodes };
        let program = Compiler::compile(program).unwrap();

        log::warn!("Program: {:?}", program);

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

    #[test]
    fn simple_looping_program() {
        simple_logger::init().unwrap_or(());
        let nodes: Nodes = [
            (
                999,
                AstNode {
                    node: InstructionNode::Start,
                    children: Some(vec![0]),
                },
            ),
            (
                0,
                AstNode {
                    node: InstructionNode::ScalarInt(IntegerNode { value: 4 }),
                    children: Some(vec![1]),
                },
            ),
            (
                1,
                AstNode {
                    node: InstructionNode::ScalarInt(IntegerNode { value: 1 }),
                    children: Some(vec![2]),
                },
            ),
            (
                3,
                AstNode {
                    node: InstructionNode::CopyLast,
                    children: Some(vec![4]),
                },
            ),
            (
                2,
                AstNode {
                    node: InstructionNode::Sub,
                    children: Some(vec![3]),
                },
            ),
            (
                4,
                AstNode {
                    node: InstructionNode::JumpIfTrue(JumpNode { nodeid: 5 }),
                    children: None,
                },
            ),
            (
                5,
                AstNode {
                    node: InstructionNode::CopyLast,
                    children: Some(vec![1]),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();

        let program = CompilationUnit { nodes };
        let program = Compiler::compile(program).unwrap();

        log::warn!("Program: {:?}", program);

        // Compilation was successful

        let mut vm = VM::new(());
        let exit_code = vm.run(&program).unwrap();

        assert_eq!(exit_code, 0);
        assert_eq!(vm.stack().len(), 3, "{:?}", vm.stack());

        for i in 3..=1 {
            let value = vm.stack()[i - 1];
            match value {
                Scalar::Integer(num) => assert_eq!(num as usize, i),
                _ => panic!("Invalid value in the stack"),
            }
        }
    }
}
