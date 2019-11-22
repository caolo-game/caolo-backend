use crate::{intents, model, profile, storage::Storage};
use cao_lang::prelude::*;
use caolo_api::{EntityId, Script, ScriptId};
use std::sync::{Arc, Mutex};

pub type ExecutionResult = Result<Vec<intents::Intent>, String>;

/// Must be called from a tokio runtime!
/// Returns the intents that are expected to be executed
pub fn execute_scripts(storage: &Storage) -> Vec<intents::Intent> {
    profile!("execute_scripts");

    let intents = Arc::new(Mutex::new(Vec::new()));
    {
        let intents = intents.clone();
        rayon::scope(move |s| {
            for (entityid, script) in storage.entity_table::<model::EntityScript>().iter() {
                let intents = intents.clone();
                s.spawn(move |_| {
                    match execute_single_script(entityid, script.script_id, storage) {
                        Ok(mut ints) => {
                            let mut intents = intents.lock().unwrap();
                            intents.append(&mut ints);
                        }
                        Err(e) => {
                            error!(
                                "Execution failure of script {:?} of entity {:?} {:?}",
                                entityid, script, e
                            );
                        }
                    }
                });
            }
        });
    }

    let intents = Arc::try_unwrap(intents).expect("Arc unwrap");
    intents.into_inner().expect("Mutex unwrap")
}

pub fn execute_single_script<'a>(
    entityid: EntityId,
    scriptid: ScriptId,
    storage: &'a Storage,
) -> ExecutionResult {
    profile!("execute_single_script");

    let program = storage
        .scripts_table::<Script>()
        .get_by_id(&scriptid)
        .ok_or_else(|| {
            error!("Script by ID {:?} does not exist", scriptid);
            "not found"
        })?;

    let data = ScriptExecutionData {
        intents: Vec::new(),
        storage: storage as *const _,
        entityid,
    };
    let mut vm = VM::new(data);
    crate::api::make_import().execute_imports(&mut vm);

    let program = program.compiled.ok_or_else(|| {
        error!("Script by ID {:?} was not compiled", scriptid);
        "not compiled"
    })?;
    vm.run(&program).map_err(|e| {
        error!(
            "Error while executing script {:?} of entity {:?}\n{:?}",
            scriptid, entityid, e
        );
        "runtime error"
    })?;

    Ok(vm.unwrap_aux().intents)
}

pub struct ScriptExecutionData {
    intents: Vec<intents::Intent>,
    storage: *const Storage,
    entityid: EntityId,
}

impl ScriptExecutionData {
    pub fn entityid(&self) -> EntityId {
        self.entityid
    }

    pub fn storage(&self) -> &Storage {
        unsafe { &*self.storage }
    }

    pub fn intents_mut(&mut self) -> &mut Vec<intents::Intent> {
        &mut self.intents
    }
}
