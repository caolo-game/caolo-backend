use super::EntityId;
use crate::prelude::*;
use crate::storage::Storage;

/// Spawns a bot from a spawn.
/// Removes the spawning bot from the spawn and initializes a bot in the world
pub fn spawn_bot(spawn_id: EntityId, entity_id: EntityId, storage: &mut Storage) {
    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?}",
        spawn_id, entity_id
    );

    let bot = storage
        .entity_table_mut::<super::SpawnBotComponent>()
        .delete(&entity_id)
        .expect("Spawning bot was not found");
    storage
        .entity_table_mut::<super::Bot>()
        .insert(entity_id, bot.bot);
    storage.entity_table_mut::<super::HpComponent>().insert(
        entity_id,
        crate::model::HpComponent {
            hp: 100,
            hp_max: 100,
        },
    );
    storage.entity_table_mut::<super::DecayComponent>().insert(
        entity_id,
        crate::model::DecayComponent {
            eta: 20,
            t: 100,
            hp_amount: 100,
        },
    );
    storage.entity_table_mut::<super::CarryComponent>().insert(
        entity_id,
        crate::model::CarryComponent {
            carry: 0,
            carry_max: 50,
        },
    );

    let positions = storage.entity_table_mut::<super::PositionComponent>();
    let pos = positions
        .get_by_id(&spawn_id)
        .cloned()
        .expect("Spawn should have position");
    positions.insert(entity_id, pos);

    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?} - done",
        spawn_id, entity_id
    );
}
