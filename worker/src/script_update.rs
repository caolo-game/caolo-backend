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
use redis::Commands;

pub fn update_program(storage: &mut Storage, client: &redis::Client) {
    debug!("Fetching new programs");
    let mut connection = client.get_connection().expect("Get redis conn");
    while let Ok(Some(program)) = connection
        .rpop::<_, Option<String>>("PROGRAM")
        .map_err(|e| {
            error!("Failed to GET script {:?}", e);
        })
    {
        debug!("Deserializing program");
        serde_json::from_str::<(
            caolo_api::UserId,
            caolo_api::ScriptId,
            caolo_api::CompiledProgram,
        )>(&program)
        .map_err(|e| {
            error!("Failed to deserialize script {:?}", e);
        })
        .ok()
        .map(|(user_id, script_id, program)| {
            use caolo_sim::model::ScriptComponent;

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
        });
    }
    debug!("Fetching new programs done");
}

fn update_user_bot_scripts(
    script_id: ScriptId,
    user_id: UserId,
    mut entity_scripts: UnsafeView<EntityId, model::EntityScript>,
    owned_entities: View<EntityId, model::OwnedEntity>,
) {
    for (_id, (_owner, entity_script)) in JoinIterator::new(
        owned_entities
            .iter()
            .filter(|(_id, owner)| owner.owner_id == user_id),
        unsafe { entity_scripts.as_mut().iter_mut() },
    ) {
        entity_script.script_id = script_id;
    }
}
