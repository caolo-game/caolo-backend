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
use crate::intents;
use crate::systems::execution::ScriptExecutionData;
use cao_lang::prelude::*;
use caolo_api::{self, OperationResult};
use rand::Rng;

/// Bootstrap the game API in the VM
pub fn make_import() -> ImportObject {
    unimplemented!()
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
