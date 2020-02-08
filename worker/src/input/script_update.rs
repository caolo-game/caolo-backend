use crate::protos::scripts::UpdateScript as UpdateScriptMsg;
use caolo_sim::model::ScriptComponent;
use caolo_sim::{
    self,
    model::{self, EntityId, ScriptId, UserId},
    storage::{
        views::{UnsafeView, View},
        Storage,
    },
    tables::JoinIterator,
};
use log::{debug, error};
use std::str::from_utf8;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum UpdateProgramError {
    BadUserId,
    BadScriptId,
}
type UpdateResult = Result<(), UpdateProgramError>;

/// Update all programs submitted via the PROGRAM field in the Redis storage
pub fn update_program(storage: &mut Storage, mut msg: UpdateScriptMsg) -> UpdateResult {
    debug!("Updating program {:?}", msg);

    let user_id = from_utf8(&msg.user_id)
        .map_err(|e| {
            error!("Failed to parse user_id as a utf8 string {:?}", e);
            UpdateProgramError::BadUserId
        })
        .and_then(|user_id| {
            Uuid::parse_str(user_id).map_err(|e| {
                error!("Failed to deserialize user_id {:?}", e);
                UpdateProgramError::BadUserId
            })
        })?;
    let script_id = from_utf8(&msg.script_id)
        .map_err(|e| {
            error!("Failed to parse script_id as a utf8 string {:?}", e);
            UpdateProgramError::BadScriptId
        })
        .and_then(|script_id| {
            Uuid::parse_str(script_id).map_err(|e| {
                error!("Failed to deserialize script_id {:?}", e);
                UpdateProgramError::BadScriptId
            })
        })
        .map(|id| caolo_api::ScriptId(id))?;

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
    let script_id = ScriptId(script_id);
    storage
        .scripts_table_mut::<ScriptComponent>()
        .insert_or_update(script_id, program);

    update_user_bot_scripts(
        script_id,
        UserId(user_id),
        From::from(storage as &mut _),
        From::from(storage as &_),
    );

    debug!("Updating program done");
    Ok(())
}

fn update_user_bot_scripts(
    script_id: ScriptId,
    user_id: UserId,
    mut entity_scripts: UnsafeView<EntityId, model::EntityScript>,
    owned_entities: View<EntityId, model::OwnedEntity>,
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
