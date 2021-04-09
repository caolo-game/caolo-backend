//! Methods that are exported to the Cao-lang clients
//!
//! Methods that may fail return an OperationResult
//!
#[cfg(test)]
mod tests;

pub mod bots;
pub mod find_api;
use crate::components;
use crate::geometry::Axial;
use crate::indices::{EntityId, WorldPosition};
use crate::profile;
use crate::systems::script_execution::ScriptExecutionData;
use cao_lang::{prelude::*, scalar::Scalar, traits::AutoByteEncodeProperties};
use find_api::FindConstant;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tracing::trace;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[repr(i32)]
pub enum OperationResult {
    Ok = 0,
    NotOwner = 1,
    InvalidInput = 2,
    OperationFailed = 3,
    NotInRange = 4,
    InvalidTarget = 5,
    Empty = 6,
    Full = 7,
    PathNotFound = 8,
}

impl TryFrom<Scalar> for OperationResult {
    type Error = Scalar;

    fn try_from(i: Scalar) -> Result<OperationResult, Scalar> {
        let op = match i {
            Scalar::Integer(0) => OperationResult::Ok,
            Scalar::Integer(1) => OperationResult::NotOwner,
            Scalar::Integer(2) => OperationResult::InvalidInput,
            Scalar::Integer(3) => OperationResult::OperationFailed,
            Scalar::Integer(4) => OperationResult::NotInRange,
            Scalar::Integer(5) => OperationResult::InvalidTarget,
            Scalar::Integer(6) => OperationResult::Empty,
            Scalar::Integer(7) => OperationResult::Full,
            Scalar::Integer(8) => OperationResult::PathNotFound,
            _ => {
                return Err(i);
            }
        };
        Ok(op)
    }
}

impl AutoByteEncodeProperties for OperationResult {}

impl From<OperationResult> for Scalar {
    fn from(opr: OperationResult) -> Self {
        Scalar::Integer(opr as i32)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub compiled: Option<CaoProgram>,
    pub script: CompilationUnit,
}

pub fn make_point(vm: &mut Vm<ScriptExecutionData>, x: i32, y: i32) -> Result<(), ExecutionError> {
    let point = Axial::new(x, y);
    vm.set_value(point)?;
    Ok(())
}

pub fn world_position(
    vm: &mut Vm<ScriptExecutionData>,
    rx: i32,
    ry: i32,
    x: i32,
    y: i32,
) -> Result<(), ExecutionError> {
    let room = Axial::new(rx, ry);
    let pos = Axial::new(x, y);
    let wp = WorldPosition { room, pos };

    vm.set_value(wp)?;
    Ok(())
}

pub fn console_log(
    vm: &mut Vm<ScriptExecutionData>,
    message: Pointer,
) -> Result<(), ExecutionError> {
    profile!("console_log");
    trace!("console_log");
    let message = vm.get_value_in_place::<&str>(message).ok_or_else(|| {
        trace!("console_log called with invalid message");
        ExecutionError::InvalidArgument {
            context: "console_log argument must be a string".to_string().into(),
        }
    })?;
    let entity_id = vm.get_aux().entity_id;
    let time = vm.get_aux().storage().time();

    let payload = message.to_string();
    trace!("{:?} says {}", entity_id, payload);
    vm.get_aux_mut().intents.with_log(entity_id, payload, time);

    Ok(())
}

pub fn log_scalar(vm: &mut Vm<ScriptExecutionData>, value: Scalar) -> Result<(), ExecutionError> {
    profile!("log_scalar");
    trace!("log_scalar");
    let entity_id = vm.get_aux().entity_id;
    let time = vm.get_aux().storage().time();
    let payload = match value {
        Scalar::Pointer(p) => {
            let mut out = String::with_capacity(64);
            if let Some(val) = vm.get_object_properties(p) {
                val.write_debug(&mut out);
            } else {
                use std::fmt::Write;
                write!(out, "invalid pointer").unwrap()
            }
            out
        }
        Scalar::Null => "null".to_string(),
        Scalar::Integer(i) => i.to_string(),
        Scalar::Floating(f) => f.to_string(),
    };
    trace!("{:?} says: {}", entity_id, payload);
    vm.get_aux_mut().intents.with_log(entity_id, payload, time);
    Ok(())
}

/// Holds data about a function
pub struct FunctionRow {
    pub desc: SubProgram<'static>,
    pub fo: Box<dyn VmFunction<ScriptExecutionData>>,
}

impl std::fmt::Debug for FunctionRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionRow of {:?}", self.desc,)
    }
}

#[derive(Debug)]
pub struct Schema {
    imports: Vec<FunctionRow>,
}

impl Schema {
    pub fn imports(&self) -> &[FunctionRow] {
        &self.imports
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.imports.iter().map(|fr| fr.desc.name)
    }

    pub fn execute_imports(self, vm: &mut Vm<ScriptExecutionData>) {
        for fr in self.imports {
            vm.register_function(fr.desc.name, move |vm: &mut Vm<_>| fr.fo.call(vm));
        }
    }
}

/// Bootstrap the game API in the Vm
pub fn make_import() -> Schema {
    Schema {
        imports: vec![
            FunctionRow {
                desc: subprogram_description!(
                    "Console Log",
                    "Log a string",
                    SubProgramType::Function,
                    [String],
                    [],
                    []
                ),
                fo: Box::new(into_f1(console_log)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Log Scalar",
                    "Log a scalar value",
                    SubProgramType::Function,
                    [Scalar],
                    [],
                    []
                ),
                fo: Box::new(into_f1(log_scalar)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Mine",
                    "Mine the target resource",
                    SubProgramType::Function,
                    [EntityId],
                    [OperationResult],
                    []
                ),
                fo: Box::new(into_f1(bots::mine_resource)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Approach Entity",
                    "Move the bot to the given Entity",
                    SubProgramType::Function,
                    [EntityId],
                    [OperationResult],
                    []
                ),
                fo: Box::new(into_f1(bots::approach_entity)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Move to position",
                    "Move the bot to the given Axial",
                    SubProgramType::Function,
                    [Axial],
                    [OperationResult],
                    []
                ),
                fo: Box::new(into_f1(bots::move_bot_to_position)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Make Point",
                    "Create a point from x and y coordinates",
                    SubProgramType::Function,
                    [i32, i32],
                    [Axial],
                    []
                ),
                fo: Box::new(into_f2(make_point)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "WorldPosition",
                    "Create a WorldPosition from coordinates: [room.x, room.y, x, y]",
                    SubProgramType::Function,
                    [i32, i32, i32, i32],
                    [Axial],
                    []
                ),
                fo: Box::new(into_f4(world_position)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Find Closest",
                    "Find an object of type `FindConstant`, closest to the current entity",
                    SubProgramType::Function,
                    [FindConstant],
                    [OperationResult, EntityId],
                    []
                ),
                fo: Box::new(into_f1(find_api::find_closest_by_range)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Unload",
                    "Unload resources",
                    SubProgramType::Function,
                    [u16, components::Resource, EntityId],
                    [OperationResult],
                    []
                ),
                fo: Box::new(into_f3(bots::unload)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Parse Find Constant",
                    "Converts string literal to a find constant",
                    SubProgramType::Function,
                    [String],
                    [FindConstant],
                    []
                ),
                fo: Box::new(into_f1(find_api::parse_find_constant)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "Melee attack",
                    "Attempts to strike the target entity",
                    SubProgramType::Function,
                    [EntityId],
                    [OperationResult],
                    []
                ),
                fo: Box::new(into_f1(bots::melee_attack)),
            },
        ],
    }
}
