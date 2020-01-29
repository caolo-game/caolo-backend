//! The public API of the game. Used in the communication layer between the simulation and clients.
//!
#[macro_use]
extern crate serde_derive;

pub mod bots;
pub mod messages;
pub mod pathfinding;
pub mod point;
pub mod point3;
pub mod resources;
pub mod structures;
pub mod terrain;
pub mod user;

pub use cao_lang::prelude::*;
pub use messages::*;

use std::convert::TryFrom;

pub type EntityId = u64;
pub type UserId = uuid::Uuid;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScriptId(pub uuid::Uuid);

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[repr(i32)]
pub enum OperationResult {
    Ok = 0,
    NotOwner = -1,
    InvalidInput = -2,
    OperationFailed = -3,
    NotInRange = -4,
    InvalidTarget = -5,
    Empty = -6,
    Full = -7,
}

impl TryFrom<i32> for OperationResult {
    type Error = i32;

    fn try_from(i: i32) -> Result<OperationResult, i32> {
        let op = match i {
            0 => OperationResult::Ok,
            -1 => OperationResult::NotOwner,
            -2 => OperationResult::InvalidInput,
            -3 => OperationResult::OperationFailed,
            -4 => OperationResult::NotInRange,
            -5 => OperationResult::InvalidTarget,
            -6 => OperationResult::Empty,
            -7 => OperationResult::Full,
            _ => {
                return Err(i);
            }
        };
        Ok(op)
    }
}

impl cao_lang::traits::AutoByteEncodeProperties for OperationResult {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub compiled: Option<CompiledProgram>,
    pub script: CompilationUnit,
}
