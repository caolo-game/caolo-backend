use super::parse_uuid;
use crate::protos::scripts::UpdateEntityScript as UpdateEntityScriptMsg;
use crate::protos::scripts::UpdateScript as UpdateScriptMsg;
use caolo_sim::model::components::{EntityScript, OwnedEntity, ScriptComponent};
use caolo_sim::prelude::*;
use caolo_sim::{
    self,
    model::{self, EntityId, ScriptId, UserId},
    tables::JoinIterator,
};
use log::{debug, error};

#[derive(Debug, Clone)]
pub enum UpdateProgramError {
    BadUserId,
    BadScriptId,
    Unauthorized,
}
type UpdateResult = Result<(), UpdateProgramError>;

/// Update all programs submitted via the PROGRAM field in the Redis storage
pub fn update_program(storage: &mut World, mut msg: UpdateScriptMsg) -> UpdateResult {
    debug!("Updating program {:?}", msg);

    let user_id = parse_uuid(&msg.user_id).map_err(|e| {
        error!("Failed to parse user_id  {:?}", e);
        UpdateProgramError::BadUserId
    })?;
    let script_id = parse_uuid(&msg.script_id)
        .map_err(|e| {
            error!("Failed to parse script_id {:?}", e);
            UpdateProgramError::BadScriptId
        })
        .map(model::ScriptId)?;

    let program = msg.take_compiled_script();
    let program = cao_lang::CompiledProgram {
        bytecode: program.bytecode,
        labels: program
            .labels
            .into_iter()
            .map(|(id, label)| {
                let label = cao_lang::Label::new(label.block, label.myself);
                (id, label)
            })
            .collect(),
    };

    debug!("Inserting new program for user {} {:?}", user_id, script_id);

    let program = ScriptComponent(program);
    unsafe {
        storage
            .unsafe_view::<ScriptId, ScriptComponent>()
            .as_mut()
            .insert_or_update(script_id, program);
    }

    update_user_bot_scripts(
        script_id,
        UserId(user_id),
        FromWorldMut::new(storage as &mut _),
        FromWorld::new(storage as &_),
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
    let entity_scripts = unsafe { entity_scripts.as_mut().iter_mut() };
    let join = JoinIterator::new(
        owned_entities
            .iter()
            .filter(|(_id, owner)| owner.owner_id == user_id),
        entity_scripts,
    );
    for (_id, (_owner, entity_script)) in join {
        entity_script.script_id = script_id;
    }
}

pub fn update_entity_script(storage: &mut World, msg: UpdateEntityScriptMsg) -> UpdateResult {
    let entity_id = EntityId(msg.entity_id);
    let user_id = parse_uuid(&msg.user_id)
        .map_err(|e| {
            error!("Failed to parse user_id {:?}", e);
            UpdateProgramError::BadUserId
        })
        .map(model::UserId)?;

    let owned_entities_table: View<EntityId, OwnedEntity> = storage.view();

    owned_entities_table
        .get_by_id(&entity_id)
        .ok_or_else(|| UpdateProgramError::Unauthorized)
        .and_then(|owner| {
            if owner.owner_id != user_id {
                Err(UpdateProgramError::Unauthorized)
            } else {
                Ok(owner)
            }
        })?;

    let mut scripts_table: UnsafeView<EntityId, EntityScript> = storage.unsafe_view();
    let script_id = parse_uuid(&msg.script_id)
        .map_err(|e| {
            error!("Failed to parse script_id {:?}", e);
            UpdateProgramError::BadScriptId
        })
        .map(model::ScriptId)?;
    unsafe {
        scripts_table
            .as_mut()
            .insert_or_update(entity_id, EntityScript { script_id });
    }
    Ok(())
}
