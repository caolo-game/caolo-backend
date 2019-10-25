use crate::{intents, model, profile, storage::Storage, UserId};
use cao_lang::prelude::*;
use caolo_api::{Script, ScriptId};
use rayon::prelude::*;

pub type ExecutionResult = Result<Vec<intents::Intent>, String>;

/// Must be called from a tokio runtime!
/// Returns the intents that are expected to be executed
pub fn execute_scripts(storage: &Storage) -> Vec<intents::Intent> {
    profile!("execute_scripts");

    unimplemented!()
}

pub fn execute_single_script(id: ScriptId, storage: &Storage) -> ExecutionResult {
    profile!("execute_single_script");

    unimplemented!();
}
