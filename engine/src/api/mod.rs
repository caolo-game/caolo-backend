//! Methods that are exported to the WASM clients
//!
//! Methods that may fail return an OperationResult or the length of the result in bytes.
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
    ($name: ident) => {
        (
            stringify!($name),
            FunctionObject::new(FunctionWrapper::new($name)),
        )
    };
}

pub fn console_log(
    vm: &mut VM<ScriptExecutionData>,
    message: TPointer,
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    let message: String = vm.get_value(message).ok_or_else(|| {
        error!("console_log called with invalid message");
        ExecutionError::InvalidArgument
    })?;

    debug!(
        "Console log EntityId[{:?}] : {}",
        vm.get_aux().entityid(),
        message
    );

    Ok(0)
}

pub fn say_hi(
    vm: &mut VM<ScriptExecutionData>,
    _: (),
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    debug!("Entity [{:?}] says hi", vm.get_aux().entityid(),);

    Ok(0)
}

pub fn log_scalar(
    vm: &mut VM<ScriptExecutionData>,
    value: Scalar,
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    debug!("Entity [{:?}] says {:?}", vm.get_aux().entityid(), value);
    Ok(0)
}

/// Bootstrap the game API in the VM
pub fn make_import() -> ImportObject {
    ImportObject {
        imports: vec![
            make_import!(console_log),
            make_import!(say_hi),
            make_import!(log_scalar),
        ],
    }
}

pub struct ImportObject {
    imports: Vec<(&'static str, FunctionObject<ScriptExecutionData>)>,
}

impl ImportObject {
    pub fn imports(&self) -> &[(&'static str, FunctionObject<ScriptExecutionData>)] {
        &self.imports
    }

    pub fn keys(&self) -> impl Iterator<Item = &&'static str> {
        self.imports.iter().map(|(k, _)| k)
    }

    pub fn execute_imports(self, vm: &mut VM<ScriptExecutionData>) {
        for (k, v) in self.imports {
            vm.register_function_obj(k, v);
        }
    }
}
