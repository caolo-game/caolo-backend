//!
//!
pub mod compiler;
pub mod instruction;
pub mod prelude;
pub mod scalar;
pub mod traits;
pub mod vm;

use crate::compiler::NodeId;
use crate::instruction::Instruction;
use arrayvec::ArrayString;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TPointer = i32;

pub const MAX_INPUT_PER_NODE: usize = 8;
pub const INPUT_STR_LEN: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN]>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompiledProgram {
    pub bytecode: Vec<u8>,
    /// Label: [block, self]
    pub labels: HashMap<NodeId, [usize; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    UnexpectedEndOfInput,
    ExitCode(i32),
    InvalidLabel,
    InvalidInstruction,
    InvalidArgument,
    FunctionNotFound(String),
    Unimplemented,
    OutOfMemory,
    MissingArgument,
    Timeout,
}

/// Metadata about a node in the program.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NodeDescription {
    pub name: &'static str,
    pub desc: &'static str,
    /// Human readable descriptions of the output
    pub output: &'static str,
    /// Human readable descriptions of inputs
    pub inputs: Vec<&'static str>,
}

impl std::fmt::Debug for NodeDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function name: {} inputs: {} output: {}",
            self.name,
            self.inputs[..].join(", "),
            self.output
        )
    }
}

#[macro_export]
macro_rules! make_input_desc {
    ($head: ty) => {
        vec![ <$head as ByteEncodeProperties>::displayname() ]
    };

    ([$($result:expr),*], $head: ty) => {
        vec![
        $($result),*
        , <$head as ByteEncodeProperties>::displayname()
        ]
    };

    ([$($result:expr),*], $head: ty, $($tail: ty),*) => {
        make_input_desc!(
            [
            $($result),*
            , <$head as ByteEncodeProperties>::displayname()
            ],
            $($tail)*
        )
    };

    ($head:ty, $($tail: ty),*) => {
        make_input_desc!(
            [ <$head as ByteEncodeProperties>::displayname() ],
            $($tail),*
        )
    };

    ([$($result:expr),*]) =>{
        vec![$($result),*]
    };
}

#[macro_export]
macro_rules! make_node_desc {
    ($name: path, $description: expr, [$($inputs: ty),*], $output: ty) => {
        {
            use cao_lang::traits::ByteEncodeProperties;
        NodeDescription {
            name: stringify!($name),
            desc: $description,
            inputs: make_input_desc!($($inputs),*) ,
            output: <$output as ByteEncodeProperties>::displayname(),
        }
        }
    };
}
