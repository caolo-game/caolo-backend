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
use cao_lang::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};
use tracing::{error, trace};

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

impl TryFrom<Value> for OperationResult {
    type Error = Value;

    fn try_from(i: Value) -> Result<OperationResult, Value> {
        let op = match i {
            Value::Integer(0) => OperationResult::Ok,
            Value::Integer(1) => OperationResult::NotOwner,
            Value::Integer(2) => OperationResult::InvalidInput,
            Value::Integer(3) => OperationResult::OperationFailed,
            Value::Integer(4) => OperationResult::NotInRange,
            Value::Integer(5) => OperationResult::InvalidTarget,
            Value::Integer(6) => OperationResult::Empty,
            Value::Integer(7) => OperationResult::Full,
            Value::Integer(8) => OperationResult::PathNotFound,
            _ => {
                return Err(i);
            }
        };
        Ok(op)
    }
}

impl From<OperationResult> for Value {
    fn from(opr: OperationResult) -> Self {
        Value::Integer(opr as i64)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub compiled: Option<CaoProgram>,
    pub script: CaoIr,
}

pub fn console_log(vm: &mut Vm<ScriptExecutionData>, message: Value) -> Result<(), ExecutionError> {
    profile!("console_log");
    trace!("console_log");
    let message = value_to_string(vm, message)?;
    let entity_id = vm.get_aux().entity_id;
    let time = vm.get_aux().storage().time();

    let payload = message.to_string();
    trace!("{:?} says {}", entity_id, payload);
    vm.get_aux_mut()
        .intents
        .with_log(entity_id, payload.as_str(), time);

    Ok(())
}

fn value_to_string(vm: &Vm<ScriptExecutionData>, value: Value) -> Result<String, ExecutionError> {
    use std::fmt::Write;

    let pl = match value {
        Value::String(p) => {
            let s = unsafe {
                vm.get_str(p)
                    .ok_or(ExecutionError::InvalidArgument { context: None })?
            };
            s.to_string()
        }
        Value::Object(p) => {
            let mut pl = String::with_capacity(256);
            pl.push('{');
            let has_value = unsafe {
                let table = &*p;
                for (hash, value) in table.iter() {
                    write!(pl, "\n\t{:?} {}", hash, value_to_string(vm, *value)?).map_err(
                        |err| {
                            error!("Failed to write value {:?}", err);
                            ExecutionError::TaskFailure("Internal Error".to_string())
                        },
                    )?;
                }
                !table.is_empty()
            };
            if has_value {
                pl.push(' ')
            } else {
                pl.push('\n');
            }
            pl.push('}');
            pl
        }
        Value::Nil => "Nil".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Floating(f) => f.to_string(),
    };
    Ok(pl)
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

/// Takes a Cao-Lang Object (FieldTable) and reads a WorldPosition from the fields:
/// - `rq` = room.q = Q component of the room id
/// - `rr` = room.r = R component of the room id
/// - `q`  = pos.q  = Q component of the room position
/// - `r`  = pos.r  = R component of the room position
pub fn parse_world_pos(point: &FieldTable) -> Result<WorldPosition, ExecutionError> {
    let rq = _get_parse_coordinate(point, "rq")?;
    let rr = _get_parse_coordinate(point, "rr")?;
    let q = _get_parse_coordinate(point, "q")?;
    let r = _get_parse_coordinate(point, "r")?;

    Ok(WorldPosition {
        room: Axial::new(rq, rr),
        pos: Axial::new(q, r),
    })
}

fn _get_parse_coordinate(point: &FieldTable, key: &str) -> Result<i32, ExecutionError> {
    let rq = point
        .get(Key::from_str(key).unwrap())
        .copied()
        .unwrap_or_default();
    let rq: i64 = rq
        .try_into()
        .map_err(|_| ExecutionError::invalid_argument("rq was not an integer".to_string()))?;
    rq.try_into().map_err(|_| {
        ExecutionError::invalid_argument("rq is not a valid coordinate value!".to_string())
    })
}

/// Bootstrap the game API in the Vm
pub fn make_import() -> Schema {
    Schema {
        imports: vec![
            FunctionRow {
                desc: subprogram_description!(
                    "console_log",
                    "Log a string",
                    SubProgramType::Function,
                    ["Text"],
                    [],
                    []
                ),
                fo: Box::new(into_f1(console_log)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "mine",
                    "Mine the target resource",
                    SubProgramType::Function,
                    ["EntityId"],
                    ["OperationResult"],
                    []
                ),
                fo: Box::new(into_f1(bots::mine_resource)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "approach_entity",
                    "Move the bot to the given Entity",
                    SubProgramType::Function,
                    ["EntityId"],
                    ["OperationResult"],
                    []
                ),
                fo: Box::new(into_f1(bots::approach_entity)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "move_to_position",
                    "Move the bot to the given Axial",
                    SubProgramType::Function,
                    ["Axial coordinate"],
                    ["OperationResult"],
                    []
                ),
                fo: Box::new(into_f1(bots::move_bot_to_position)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "find_closest",
                    "Find an object of type `FindConstant`, closest to the current entity. Returns `Nil` if no such entity is found",
                    SubProgramType::Function,
                    ["FindConstant"],
                    ["EntityId"],
                    []
                ),
                fo: Box::new(into_f1(find_api::find_closest_by_range)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "unload",
                    "Unload resources",
                    SubProgramType::Function,
                    ["Integer", "Resource", "EntityId"],
                    ["OperationResult"],
                    []
                ),
                fo: Box::new(into_f3(bots::unload)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "parse_find_constant",
                    "Converts string literal to a find constant",
                    SubProgramType::Function,
                    ["Text"],
                    ["FindConstant"],
                    []
                ),
                fo: Box::new(into_f1(find_api::parse_find_constant)),
            },
            FunctionRow {
                desc: subprogram_description!(
                    "melee_attack",
                    "Attempts to strike the target entity",
                    SubProgramType::Function,
                    ["EntityId"],
                    ["OperationResult"],
                    []
                ),
                fo: Box::new(into_f1(bots::melee_attack)),
            },
        ],
    }
}
