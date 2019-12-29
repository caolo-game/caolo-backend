//! Methods that are exported to the WASM clients
//!
//! Methods that may fail return an OperationResult
//!
mod bots;
mod resources;
mod structures;
pub use self::bots::*;
pub use self::resources::*;
pub use self::structures::*;
use crate::model::Point;
use crate::systems::execution::ScriptExecutionData;
use cao_lang::prelude::*;
use cao_lang::traits::ByteEncodeProperties;
use caolo_api::OperationResult;

pub fn make_point(
    vm: &mut VM<ScriptExecutionData>,
    (x, y): (i32, i32),
    output: TPointer,
) -> Result<usize, ExecutionError> {
    let point = Point::new(x, y);
    Ok(vm.set_value_at(output, point))
}

pub fn console_log(
    vm: &mut VM<ScriptExecutionData>,
    message: TPointer,
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    let entityid = vm.get_aux().entityid();
    let time = vm.get_aux().storage().time();
    let message: String = vm.get_value(message).ok_or_else(|| {
        error!("console_log called with invalid message");
        ExecutionError::InvalidArgument
    })?;

    let payload = format!("Console log EntityId[{:?}] : {}", entityid, message);
    debug!("{}", payload);
    vm.get_aux_mut()
        .intents_mut()
        .push(crate::intents::Intent::new_log(entityid, payload, time));

    Ok(0)
}

pub fn log_scalar(
    vm: &mut VM<ScriptExecutionData>,
    value: Scalar,
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    let entityid = vm.get_aux().entityid();
    let time = vm.get_aux().storage().time();
    let payload = format!("Entity [{:?}] says {:?}", entityid, value);
    debug!("{}", payload);
    vm.get_aux_mut()
        .intents_mut()
        .push(crate::intents::Intent::new_log(entityid, payload, time));
    Ok(0)
}

/// Holds data about a function
pub struct FunctionRow {
    pub desc: NodeDescription<'static>,
    pub fo: FunctionObject<ScriptExecutionData>,
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

    pub fn execute_imports(self, vm: &mut VM<ScriptExecutionData>) {
        for fr in self.imports {
            vm.register_function_obj(fr.desc.name, fr.fo);
        }
    }
}

/// Bootstrap the game API in the VM
pub fn make_import() -> Schema {
    Schema {
        imports: vec![
            FunctionRow {
                desc: make_node_desc!(console_log, "Log a string", [String], ()),
                fo: FunctionObject::new(FunctionWrapper::new(console_log)),
            },
            FunctionRow {
                desc: make_node_desc!(log_scalar, "Log a scalar value", [Scalar], ()),
                fo: FunctionObject::new(FunctionWrapper::new(log_scalar)),
            },
            FunctionRow {
                desc: make_node_desc!(
                    bots::move_bot,
                    "Move the bot to the given Point",
                    [Point],
                    OperationResult
                ),
                fo: FunctionObject::new(FunctionWrapper::new(bots::move_bot)),
            },
            FunctionRow {
                desc: make_node_desc!(
                    make_point,
                    "Create a point from x and y coordinates",
                    [Scalar, Scalar],
                    Point
                ),
                fo: FunctionObject::new(FunctionWrapper::new(make_point)),
            },
        ],
    }
}
