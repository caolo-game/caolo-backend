//! Methods that are exported to the WASM clients
//!
//! Methods that may fail return an OperationResult
//!
mod bots;
mod pathfinding;
mod resources;
mod structures;
pub use self::bots::*;
pub use self::pathfinding::*;
pub use self::resources::*;
pub use self::structures::*;
use crate::systems::execution::ScriptExecutionData;
use cao_lang::prelude::*;

macro_rules! make_import {
    ($name: path, $description: expr) => {
        (
            stringify!($name),
            $description,
            FunctionObject::new(FunctionWrapper::new($name)),
        )
    };
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

/// Bootstrap the game API in the VM
pub fn make_import() -> ImportObject {
    ImportObject {
        imports: vec![
            make_import!(console_log, "Log a string"),
            make_import!(log_scalar, "Log the topmost scalar on the stack"),
            make_import!(bots::move_bot, "Move the current bot to the Point"),
        ],
    }
}

pub struct ImportObject {
    imports: Vec<(
        &'static str,
        &'static str,
        FunctionObject<ScriptExecutionData>,
    )>,
}

impl ImportObject {
    pub fn imports(
        &self,
    ) -> &[(
        &'static str,
        &'static str,
        FunctionObject<ScriptExecutionData>,
    )] {
        &self.imports
    }

    pub fn keys(&self) -> impl Iterator<Item = &&'static str> {
        self.imports.iter().map(|(k, _, _)| k)
    }

    pub fn execute_imports(self, vm: &mut VM<ScriptExecutionData>) {
        for (k, _, v) in self.imports {
            vm.register_function_obj(k, v);
        }
    }
}
