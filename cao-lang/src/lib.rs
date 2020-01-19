//! ## Programs
//!
//! Programs are composed of subprograms. A subprogram consumes inputs and produces outputs.
//! Subprograms will always consume from the top of the stack downwards and push their outputs to
//! the stack. This means that subprogram composition is not a commutative operation. (Consider
//! subprograms A, B and C. Then the composition ABC is not the same as BAC if A != B. )
//!
//! Programs passed to the `Compiler` must contain a `Start` node. Execution will begin at the
//! first `Start` node.
//!
//! Example (Sub) Program serialized as JSON
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
mod macros;

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

/// Metadata about a subprogram in the program.
/// Subprograms consume their inputs and produce outputs.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SubProgram<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    /// Human readable descriptions of the output
    pub output: Vec<&'a str>,
    /// Human readable descriptions of inputs
    pub input: Vec<&'a str>,
}

impl<'a> std::fmt::Debug for SubProgram<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function name: {} inputs: {} outputs: {}",
            self.name,
            self.input[..].join(", "),
            self.output[..].join(", ")
        )
    }
}

#[macro_export]
macro_rules! subprogram_description {
    ($name: path, $description: expr, [$($inputs: ty),*], [$($outputs: ty),*]) => {
        SubProgram {
            name: stringify!($name),
            desc: $description,
            input: subprogram_description!(input $($inputs),*) ,
            output: subprogram_description!(input $($outputs),*),
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
        subprogram_description!(
            input
            [
            $($result),*
            , <$head as ByteEncodeProperties>::displayname()
            ],
            $($tail)*
        )
    };

    (input $head:ty, $($tail: ty),*) => {
        subprogram_description!(
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
