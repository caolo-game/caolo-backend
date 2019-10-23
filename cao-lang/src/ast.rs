use crate::{Instruction, Value};
use arrayvec::{ArrayString, ArrayVec};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Inputs = HashMap<NodeId, ArrayVec<[NodeId; 16]>>;
pub type Nodes = HashMap<NodeId, AstNode>;
/// Value of a node if any
pub type Values = HashMap<NodeId, Value>;
/// String of a node if any
pub type Strings = HashMap<NodeId, InputString>;

const INPUT_STR_LEN: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN]>;

impl crate::ByteEncodeProperties for InputString {
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
    instruction: Instruction,
}

impl AstNode {
    pub fn new(instruction: Instruction) -> Self {
        Self { instruction }
    }
}

/// The accepted number of inputs of an instruction
/// None if unspecified
pub fn input_per_instruction(inst: Instruction) -> Option<u8> {
    use Instruction::*;
    match inst {
        AddInt | SubInt | AddFloat | SubFloat | Mul | MulFloat | Div | DivFloat => Some(2),
        LiteralInt | LiteralFloat | LiteralPtr => Some(1),
        Pass | CopyLast => Some(0),
        Call | LiteralArray => None,
    }
}

/// Single unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    nodes: Nodes,
    inputs: Inputs,
    values: Values,
    strings: Strings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProgram {
    pub leafid: NodeId,
    pub bytecode: Vec<u8>,
}

pub struct Compiler {
    unit: CompilationUnit,
}

impl Compiler {
    pub fn compile(unit: CompilationUnit) -> Result<Vec<CompiledProgram>, String> {
        if unit.nodes.is_empty() {
            return Err("Can not compile program with no entry point!".to_owned());
        }
        let compiler = Compiler { unit };
        let mut todo: Vec<NodeId> = compiler
            .unit
            .nodes
            .iter()
            .filter(|(k, v)| {
                //
                true
            })
            .map(|(k, _)| k)
            .cloned()
            .collect();

        let mut compiled_programs = Vec::with_capacity(4);
        for nodeid in todo {
            // TODO: todo should be leaf nodes
        }

        Ok(compiled_programs)
    }

    fn compile_node(&mut self, node: NodeId) -> Result<CompiledProgram, String> {
        let mut compiled = CompiledProgram {
            bytecode: Vec::new(),
            leafid: node,
        };
        self.process_node(node, &mut compiled.bytecode)?;

        Ok(compiled)
    }

    fn process_node(&mut self, nodeid: NodeId, bytes: &mut Vec<u8>) -> Result<(), String> {
        Compiler::validate_node(nodeid, &mut self.unit)?;

        for nodeid in self.unit.inputs[&nodeid].clone().into_iter() {
            self.process_node(nodeid, bytes)?;
        }
        let node = &self.unit.nodes[&nodeid];
        {
            use crate::traits::ByteEncodeProperties;
            use Instruction::*;
            match node.instruction {
                Call => {
                    bytes.push(node.instruction as u8);
                    bytes.append(&mut self.unit.strings[&nodeid].encode());
                }
                LiteralArray | LiteralPtr | LiteralFloat | LiteralInt => {
                    bytes.push(node.instruction as u8);
                    bytes.append(&mut self.unit.values[&nodeid].encode());
                }
                _ => bytes.push(node.instruction as u8),
            }
        }
        Ok(())
    }

    pub fn validate_node(node: NodeId, cu: &CompilationUnit) -> Result<(), String> {
        if let Some(n) = input_per_instruction(cu.nodes[&node].instruction) {
            let n_inputs = cu.inputs.get(&node).map(|x| x.len()).unwrap_or(0)
                + cu.strings.get(&node).map(|_| 1).unwrap_or(0);
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
