use crate::protos::cao_script::{
    SetDefaultScriptCommand, UpdateEntityScriptCommand, UpdateScriptCommand,
};
use caolo_sim::{self, prelude::*, tables::JoinIterator};
use thiserror::Error;
use tracing::{debug, error};

#[derive(Debug, Error)]
pub enum UpdateProgramError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Missing expected field {0}")]
    MissingField(&'static str),
    #[error("Failed to parse uuid {0}")]
    UuidError(anyhow::Error),
    #[error("Failed to compile the script {0}")]
    CompilationError(cao_lang::prelude::CompilationError),
    #[error("Failed to deserialize the compilation unit {0}")]
    CuDeserializationError(serde_json::Error),
}

type UpdateResult = Result<(), UpdateProgramError>;

pub fn update_program(storage: &mut World, msg: &UpdateScriptCommand) -> UpdateResult {
    debug!("Updating program");

    let user_id = msg
        .user_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("user_id"))?
        .data
        .as_slice();
    let user_id =
        uuid::Uuid::from_slice(user_id).map_err(|err| UpdateProgramError::UuidError(err.into()))?;

    let script_id = msg
        .script_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("script_id"))?
        .data
        .as_slice();
    let script_id = uuid::Uuid::from_slice(script_id)
        .map_err(|err| UpdateProgramError::UuidError(err.into()))?;

    debug!("Inserting new program for user {} {}", user_id, script_id);

    let user_id = UserId(user_id);
    let script_id = ScriptId(script_id);

    let cu = msg
        .compilation_unit
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("compilation_unit"))?
        .encoded
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("compilation_unit.encoded"))?
        .value
        .as_slice();

    let compilation_unit: cao_lang::compiler::CaoIr =
        serde_json::from_slice(cu).map_err(UpdateProgramError::CuDeserializationError)?;

    let program = cao_lang::prelude::compile(compilation_unit.clone(), None)
        .map_err(UpdateProgramError::CompilationError)?;

    let program = CompiledScriptComponent(program);
    storage
        .unsafe_view::<ScriptId, CompiledScriptComponent>()
        .insert_or_update(script_id, program);
    // store the raw CaoIr to be queried by clients
    storage
        .unsafe_view::<ScriptId, CaoIrComponent>()
        .insert_or_update(script_id, CaoIrComponent(compilation_unit));

    update_user_bot_scripts(
        script_id,
        user_id,
        FromWorldMut::from_world_mut(storage as &mut _),
        FromWorld::from_world(storage as &_),
    );

    debug!("Updating program done");
    Ok(())
}

fn update_user_bot_scripts(
    script_id: ScriptId,
    user_id: UserId,
    mut entity_scripts: UnsafeView<EntityId, EntityScript>,
    owned_entities: View<EntityId, OwnedEntity>,
) {
    let entity_scripts = entity_scripts.iter_mut();
    let join = JoinIterator::new(
        owned_entities
            .iter()
            .filter(|(_id, owner)| owner.owner_id == user_id),
        entity_scripts,
    );
    for (_id, (_owner, entity_script)) in join {
        entity_script.0 = script_id;
    }
}

pub fn update_entity_script(storage: &mut World, msg: &UpdateEntityScriptCommand) -> UpdateResult {
    let user_id = msg
        .user_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("user_id"))?
        .data
        .as_slice();
    let user_id =
        uuid::Uuid::from_slice(user_id).map_err(|err| UpdateProgramError::UuidError(err.into()))?;

    let entity_id = EntityId(msg.entity_id);

    let owned_entities_table: View<EntityId, OwnedEntity> = storage.view();

    owned_entities_table
        .get_by_id(entity_id)
        .ok_or(UpdateProgramError::Unauthorized)
        .and_then(|owner| {
            if owner.owner_id.0 != user_id {
                Err(UpdateProgramError::Unauthorized)
            } else {
                Ok(owner)
            }
        })?;

    let script_id = msg
        .script_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("script_id"))?
        .data
        .as_slice();
    let script_id = uuid::Uuid::from_slice(script_id)
        .map_err(|err| UpdateProgramError::UuidError(err.into()))?;
    let script_id = ScriptId(script_id);

    let mut scripts_table: UnsafeView<EntityId, EntityScript> = storage.unsafe_view();
    scripts_table.insert_or_update(entity_id, EntityScript(script_id));
    Ok(())
}

pub fn set_default_script(storage: &mut World, msg: &SetDefaultScriptCommand) -> UpdateResult {
    let user_id = msg
        .user_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("user_id"))?
        .data
        .as_slice();
    let script_id = msg
        .script_id
        .as_ref()
        .ok_or(UpdateProgramError::MissingField("script_id"))?
        .data
        .as_slice();

    let user_id =
        uuid::Uuid::from_slice(user_id).map_err(|err| UpdateProgramError::UuidError(err.into()))?;
    let script_id = uuid::Uuid::from_slice(script_id)
        .map_err(|err| UpdateProgramError::UuidError(err.into()))?;

    let user_id = UserId(user_id);
    let script_id = ScriptId(script_id);

    let script = EntityScript(script_id);

    let mut user_default_script: UnsafeView<UserId, EntityScript> = storage.unsafe_view();
    user_default_script.insert_or_update(user_id, script);

    Ok(())
}
