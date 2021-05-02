use crate::{
    components::{game_config::GameConfig, CompiledScriptComponent, EntityScript, OwnedEntity},
    diagnostics::Diagnostics,
    indices::{ConfigKey, EntityId, ScriptId, UserId},
    intents::*,
    prelude::{EmptyKey, World},
    profile,
    storage::views::{FromWorld, UnwrapView},
};
use cao_lang::prelude::*;
use futures::StreamExt;
use std::mem::replace;
use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
};
use thiserror::Error;
use tracing::{debug, trace, warn};

pub type ExecutionResult = Result<BotIntents, ExecutionError>;

#[derive(Debug, Error, Clone)]
pub enum ExecutionError {
    #[error("{0:?} was not found")]
    ScriptNotFound(ScriptId),
    #[error(" {script_id:?} of {entity_id:?} failed {error:?}")]
    RuntimeError {
        script_id: ScriptId,
        entity_id: EntityId,
        error: cao_lang::prelude::ExecutionError,
    },
}

pub async fn execute_scripts(
    workload: &[(EntityId, EntityScript)],
    storage: &mut World,
) -> Result<Vec<BotIntents>, Infallible> {
    profile!("execute_scripts");

    let start = chrono::Utc::now();

    let owners_table = storage.view::<EntityId, OwnedEntity>().reborrow();

    let n_scripts = workload.len();

    let chunk_size = n_scripts.clamp(8, 256);

    debug!(
        "Executing {} scripts in chunks of {}",
        n_scripts, chunk_size
    );

    #[derive(Default)]
    struct RunResult {
        intents: Vec<BotIntents>,
        num_scripts_ran: u64,
        num_scripts_errored: u64,
    }

    let run_result = futures::stream::iter(workload)
        .chunks(chunk_size)
        .map(|entity_scripts| {
            let mut results = RunResult {
                intents: Vec::with_capacity(chunk_size),
                num_scripts_ran: 0,
                num_scripts_errored: 0,
            };
            let data = ScriptExecutionData::unsafe_default();

            let conf = UnwrapView::<ConfigKey, GameConfig>::from_world(storage);
            let mut vm = Vm::new(data).expect("Failed to initialize VM");
            vm.runtime_data.set_memory_limit(40 * 1024 * 1024);
            vm.max_instr = conf.execution_limit as u64;
            crate::scripting_api::make_import().execute_imports(&mut vm);

            for (entity_id, script) in entity_scripts {
                let owner_id = owners_table
                    .get_by_id(*entity_id)
                    .map(|OwnedEntity { owner_id }| *owner_id);

                let s = tracing::error_span!("script_execution", entity_id = entity_id.0);
                let _e = s.enter();

                vm.clear();
                match execute_single_script(*entity_id, script.0, owner_id, storage, &mut vm) {
                    Ok(ints) => results.intents.push(ints),
                    Err(err) => {
                        results.num_scripts_errored += 1;
                        debug!(
                            "Execution failure in {:?} of {:?}:\n{:?}",
                            script, entity_id, err
                        );
                    }
                }
                results.num_scripts_ran += 1;
            }
            results
        })
        .fold(RunResult::default(), |mut res, intermediate| async move {
            res.intents.extend(intermediate.intents);
            res.num_scripts_ran += intermediate.num_scripts_ran;
            res.num_scripts_errored += intermediate.num_scripts_errored;
            res
        })
        .await;

    debug!(
        "Executing scripts done. Returning {:?} intents",
        run_result.intents.len()
    );

    let mut diag = storage.unsafe_view::<EmptyKey, Diagnostics>();
    let diag: &mut Diagnostics = diag.unwrap_mut_or_default();

    diag.update_scripts(
        chrono::Utc::now() - start,
        run_result.num_scripts_ran,
        run_result.num_scripts_errored,
    );

    Ok(run_result.intents)
}

fn prepare_script_data(
    entity_id: EntityId,
    user_id: Option<UserId>,
    storage: &World,
) -> ScriptExecutionData {
    let intents = BotIntents {
        entity_id,
        ..Default::default()
    };
    ScriptExecutionData::new(storage, intents, entity_id, user_id)
}

pub fn execute_single_script<'a>(
    entity_id: EntityId,
    script_id: ScriptId,
    user_id: Option<UserId>,
    storage: &'a World,
    vm: &mut Vm<'a, ScriptExecutionData>,
) -> ExecutionResult {
    let program = storage
        .view::<ScriptId, CompiledScriptComponent>()
        .reborrow()
        .get_by_id(script_id)
        .ok_or_else(|| {
            warn!("Script by ID {:?} does not exist", script_id);
            ExecutionError::ScriptNotFound(script_id)
        })?;

    let data = prepare_script_data(entity_id, user_id, storage);
    vm.auxiliary_data = data;

    trace!("Starting script execution");

    vm.run(&program.0).map_err(|err| {
        warn!("Error while executing script {:?} {:?}", script_id, err);
        ExecutionError::RuntimeError {
            script_id,
            entity_id,
            error: err,
        }
    })?;

    let aux = replace(
        &mut vm.auxiliary_data,
        ScriptExecutionData::unsafe_default(),
    );
    trace!("Script execution completed, intents:{:?}", aux.intents);

    let intents = aux.intents;
    Ok(intents)
}

#[derive(Debug)]
pub struct ScriptExecutionData {
    pub entity_id: EntityId,
    pub user_id: Option<UserId>,
    pub intents: BotIntents,
    storage: *const World,
}

impl Display for ScriptExecutionData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.entity_id)?;
        if let Some(ref user_id) = self.user_id {
            write!(f, " UserId: {}", user_id.0)?
        }
        Ok(())
    }
}

impl ScriptExecutionData {
    /// To be used as a placeholder, do not consume
    pub fn unsafe_default() -> Self {
        Self {
            entity_id: Default::default(),
            user_id: None,
            intents: Default::default(),
            storage: std::ptr::null(),
        }
    }

    pub fn new(
        storage: &World,
        intents: BotIntents,
        entity_id: EntityId,
        user_id: Option<UserId>,
    ) -> Self {
        Self {
            storage: storage as *const _,
            intents,
            entity_id,
            user_id,
        }
    }

    pub fn storage(&self) -> &World {
        unsafe { &*self.storage }
    }
}
