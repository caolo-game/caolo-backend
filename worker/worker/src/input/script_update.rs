use super::parse_uuid;
use crate::protos::cao_commands::{
    SetDefaultScriptCommand, UpdateEntityScriptCommand, UpdateScriptCommand,
};
use anyhow::Context;
use caolo_sim::{self, prelude::*, tables::JoinIterator};
use slog::{debug, error, Logger};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateProgramError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Failed to perform the operation {0:?}")]
    Internal(anyhow::Error),
    #[error("Invalid message {0:?}")]
    BadMessage(anyhow::Error),
}
type UpdateResult = Result<(), UpdateProgramError>;

pub fn update_program(
    logger: Logger,
    storage: &mut World,
    msg: &UpdateScriptCommand,
) -> UpdateResult {
    debug!(logger, "Updating program");

    let user_id = parse_uuid(msg.get_userId()).map_err(UpdateProgramError::BadMessage)?;
    let script_id = parse_uuid(msg.get_scriptId()).map_err(UpdateProgramError::BadMessage)?;

    debug!(
        logger,
        "Inserting new program for user {} {}", user_id, script_id
    );

    let user_id = UserId(user_id);
    let script_id = ScriptId(script_id);

    let cu = msg.get_compilationUnit().get_compilationUnit().get_value();

    let compilation_unit = serde_json::from_slice(cu)
        .with_context(|| "Failed to deserialize CU")
        .map_err(UpdateProgramError::BadMessage)?;

    let program = cao_lang::prelude::compile(logger.clone(), compilation_unit, None)
        .with_context(|| "Failed to compile script")
        .map_err(UpdateProgramError::BadMessage)?;

    let program = ScriptComponent(program);
    storage
        .unsafe_view::<ScriptId, ScriptComponent>()
        .insert_or_update(script_id, program);

    update_user_bot_scripts(
        script_id,
        user_id,
        FromWorldMut::new(storage as &mut _),
        FromWorld::new(storage as &_),
    );

    debug!(logger, "Updating program done");
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
    let user_id = parse_uuid(msg.get_userId()).map_err(UpdateProgramError::Internal)?;

    let entity_id = EntityId(msg.get_entityId());

    let owned_entities_table: View<EntityId, OwnedEntity> = storage.view();

    owned_entities_table
        .get_by_id(&entity_id)
        .ok_or(UpdateProgramError::Unauthorized)
        .and_then(|owner| {
            if owner.owner_id.0 != user_id {
                Err(UpdateProgramError::Unauthorized)
            } else {
                Ok(owner)
            }
        })?;

    let script_id = parse_uuid(msg.get_scriptId()).map_err(UpdateProgramError::Internal)?;
    let script_id = ScriptId(script_id);

    let mut scripts_table: UnsafeView<EntityId, EntityScript> = storage.unsafe_view();
    scripts_table.insert_or_update(entity_id, EntityScript(script_id));
    Ok(())
}

pub fn set_default_script(storage: &mut World, msg: &SetDefaultScriptCommand) -> UpdateResult {
    let user_id = parse_uuid(msg.get_userId()).map_err(UpdateProgramError::BadMessage)?;
    let script_id = parse_uuid(msg.get_scriptId()).map_err(UpdateProgramError::BadMessage)?;

    let user_id = UserId(user_id);
    let script_id = ScriptId(script_id);

    let script = EntityScript(script_id);

    let mut user_default_script: UnsafeView<UserId, EntityScript> = storage.unsafe_view();
    user_default_script.insert_or_update(user_id, script);

    Ok(())
}
