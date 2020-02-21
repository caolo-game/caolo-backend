//! Compiles Graphs with vertices of `AstNode` into _caol-lang_ bytecode.
//! Programs must start with a `Start` instruction.
//!
#[cfg(test)]
mod tests ;
mod astnode;
use crate::{
    traits::ByteEncodeProperties, CompiledProgram, InputString, Instruction, Label, INPUT_STR_LEN,
};
pub use astnode::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::TryFrom;
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

        let fromlabel = u32::try_from(self.program.bytecode.len())
            .expect("bytecode length to fit into 32 bits");
        self.program
            .labels
            .insert(nodeid, Label::new(fromlabel, fromlabel));

        let instruction = node.node;

        match instruction {
            Pop | Equals | Less | LessOrEq | NotEquals | Exit | Start | Pass | CopyLast | Add
            | Sub | Mul | Div => {
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

