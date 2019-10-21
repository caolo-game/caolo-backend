use crate::{Instruction, Value};
use arrayvec::{ArrayString, ArrayVec};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub type Strings = HashMap<NodeId, ArrayString<[u8; 128]>>;

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
        Pass => Some(0),
        Call | LiteralArray => None,
    }
}

/// Single unit of compilation, representing a single program
/// The entry point is the node with ID 0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    nodes: Nodes,
    inputs: Inputs,
    values: Values,
    strings: Strings,
}

pub fn compile(unit: CompilationUnit) -> Result<Vec<u8>, String> {
    if unit.nodes.len() < 1 || unit.nodes.get(&0).is_none() {
        return Err("Can not compile program with no entry point!".to_owned());
    }
    unimplemented!()
}

pub fn validate_node(node: NodeId, cu: &CompilationUnit) -> Result<(), String> {
    if let Some(n) = input_per_instruction(cu.nodes[&node].instruction) {
        let n_inputs = cu.inputs[&node].len() + cu.strings[&node].len();
        if n_inputs != n as usize {
            return Err(format!(
                "Invalid number of inputs, expected {} got {}",
                n, n_inputs
            ));
        }
    }
    Ok(())
}
