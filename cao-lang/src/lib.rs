//! Example Program serialized as JSON
//! ```
//! const PROGRAM: &str = r#"{
//!     "nodes": {
//!         "0": {
//!             "node": {
//!                 "Start": null
//!             },
//!             "children": [
//!                 1
//!             ]
//!         },
//!         "1": {
//!             "node": {
//!                 "ScalarInt": {
//!                     "value": 42
//!                 }
//!             },
//!             "children": [
//!                 2
//!             ]
//!         },
//!         "2": {
//!             "node": {
//!                 "Call": {
//!                     "function": "log_scalar"
//!                 }
//!             }
//!         }
//!     }
//! }"#;
//!
//! let compilation_unit = serde_json::from_str(PROGRAM).unwrap();
//! cao_lang::compiler::Compiler::compile(compilation_unit).unwrap();
//!```
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
    TaskFailure(String),
}

/// Metadata about a node in the program.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NodeDescription<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    /// Human readable descriptions of the output
    pub output: &'a str,
    /// Human readable descriptions of inputs
    pub inputs: Vec<&'a str>,
}

impl<'a> std::fmt::Debug for NodeDescription<'a> {
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
macro_rules! make_node_desc {
    ($name: path, $description: expr, [$($inputs: ty),*], $output: ty) => {
        NodeDescription {
            name: stringify!($name),
            desc: $description,
            inputs: make_node_desc!(input $($inputs),*) ,
            output: <$output as ByteEncodeProperties>::displayname(),
        }
    };

    (input $head: ty) => {
        vec![ <$head as ByteEncodeProperties>::displayname() ]
    };

    (input [$($result:expr),*], $head: ty) => {
        vec![
        $($result),*
        , <$head as ByteEncodeProperties>::displayname()
        ]
    };

    (input [$($result:expr),*], $head: ty, $($tail: ty),*) => {
        make_node_desc!(
            input
            [
            $($result),*
            , <$head as ByteEncodeProperties>::displayname()
            ],
            $($tail)*
        )
    };

    (input $head:ty, $($tail: ty),*) => {
        make_node_desc!(
            input
            [ <$head as ByteEncodeProperties>::displayname() ],
            $($tail),*
        )
    };

    (input [$($result:expr),*]) => {
        vec![$($result),*]
    };

    (input) => {
        vec![]
    };
}
