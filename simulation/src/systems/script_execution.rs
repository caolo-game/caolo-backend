use crate::model::{self, EntityId, ScriptId, UserId};
use crate::{intents, profile, storage::Storage};
use cao_lang::prelude::*;
use std::sync::{Arc, Mutex};

pub type ExecutionResult = Result<intents::Intents, String>;

/// Must be called from a tokio runtime!
/// Returns the intents that are expected to be executed
pub fn execute_scripts(storage: &Storage) -> intents::Intents {
    profile!("execute_scripts");

    let intents = Arc::new(Mutex::new(intents::Intents::new()));
    {
        let intents = intents.clone();
        rayon::scope(move |s| {
            for (entityid, script) in storage.entity_table::<model::EntityScript>().iter() {
                let intents = intents.clone();
                s.spawn(move |_| {
                    match execute_single_script(entityid, script.script_id, storage) {
                        Ok(ints) => {
                            let mut intents = intents.lock().unwrap();
                            intents.merge(ints);
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
        .scripts_table::<model::ScriptComponent>()
        .get_by_id(&scriptid)
        .ok_or_else(|| {
            error!("Script by ID {:?} does not exist", scriptid);
            "not found"
        })?;

    let data = ScriptExecutionData {
        intents: intents::Intents::new(),
        storage: storage as *const _,
        entityid,
        current_user: Some(Default::default()), // None, // TODO
    };
    let mut vm = VM::new(data);
    crate::api::make_import().execute_imports(&mut vm);

    let program = program.0.compiled.as_ref().ok_or_else(|| {
        error!("Script by ID {:?} was not compiled", scriptid);
        "not compiled"
    })?;
    vm.run(program).map_err(|e| {
        warn!(
            "Error while executing script {:?} of entity {:?}\n{:?}",
            scriptid, entityid, e
        );
        "runtime error"
    })?;

    Ok(vm.unwrap_aux().intents)
}

pub struct ScriptExecutionData {
    intents: intents::Intents,
    storage: *const Storage,
    entityid: EntityId,
    current_user: Option<UserId>,
}

impl ScriptExecutionData {
    pub fn entityid(&self) -> EntityId {
        self.entityid
    }

    pub fn storage(&self) -> &Storage {
        unsafe { &*self.storage }
    }

    pub fn intents_mut(&mut self) -> &mut intents::Intents {
        &mut self.intents
    }

    pub fn userid(&self) -> Option<UserId> {
        self.current_user
    }
}
