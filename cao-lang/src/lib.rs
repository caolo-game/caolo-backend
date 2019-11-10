pub mod compiler;
pub mod instruction;
pub mod prelude;
pub mod scalar;
pub mod traits;
pub mod vm;

use crate::compiler::NodeId;
use crate::instruction::Instruction;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TPointer = i32;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompiledProgram {
    pub bytecode: Vec<u8>,
    pub labels: HashMap<NodeId, usize>,
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
}
