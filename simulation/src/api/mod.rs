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
use arrayvec::ArrayVec;
use cao_lang::prelude::*;
use cao_lang::traits::ByteEncodeProperties;

macro_rules! make_input_desc {
    () => {None};

    ($head: path) => {
        Some(<$head as ByteEncodeProperties>::displayname())
    };

    ($head: path, $($tail: tt),*) => {
        Some(<$head as ByteEncodeProperties>::displayname()),
        make_input_desc!($($tail)*)
    };
}

macro_rules! make_import {
    ($name: path, $description: expr, [$($inputs: path),*], $output: ty) => {
        FunctionRow {
            name: stringify!($name),
            description: $description,
            inputs: Box::new(|| [ make_input_desc!($($inputs),*) ].iter().filter_map(|x|*x).collect()),
            output: <$output as ByteEncodeProperties>::displayname(),
            fo: FunctionObject::new(FunctionWrapper::new($name)),
        }
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
pub fn make_import() -> Schema {
    Schema {
        imports: vec![
            make_import!(console_log, "Log a string", [String], ()),
            make_import!(log_scalar, "Log a scalar value", [Scalar], ()),
            make_import!(
                bots::move_bot,
                "Move the bot to the given Point",
                [crate::model::Point],
                caolo_api::OperationResult
            ),
        ],
    }
}

/// Holds data about a function
pub struct FunctionRow {
    pub name: &'static str,
    pub description: &'static str,
    /// Human readable names of inputs
    pub inputs: Box<dyn Fn() -> ArrayVec<[&'static str; cao_lang::MAX_INPUT_PER_NODE]>>,
    /// Human readable name of output
    pub output: &'static str,
    pub fo: FunctionObject<ScriptExecutionData>,
}

impl std::fmt::Debug for FunctionRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Function name: {} inputs: {} output: {}",
            self.name,
            self.inputs()[..].join(", "),
            self.output
        )
    }
}

impl FunctionRow {
    pub fn inputs(&self) -> ArrayVec<[&'static str; cao_lang::MAX_INPUT_PER_NODE]> {
        (*self.inputs)()
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
        self.imports.iter().map(|fr| fr.name)
    }

    pub fn execute_imports(self, vm: &mut VM<ScriptExecutionData>) {
        for fr in self.imports {
            vm.register_function_obj(fr.name, fr.fo);
        }
    }
}
