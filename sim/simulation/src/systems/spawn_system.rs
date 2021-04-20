//! Spawn logic consists of 3 steps:
//!
//! - Spawn Intent will add a bot spawn task to the queue if it isn't full
//! - Spawn update will first decrement time to spawn and spawn the bot if it reaches 0
//! - If time to spawn is 0 and the queue is not empty start another spawn process
//!
mod continous_spawn_system;
mod spawn_intent_system;

pub use continous_spawn_system::update as update_cont_spawns;
pub use spawn_intent_system::update as update_spawn_intents;

use crate::indices::{EntityId, UserId};
use crate::join;
use crate::profile;
use crate::storage::views::{UnsafeView, View};
use crate::tables::{JoinIterator, Table};
use crate::{components::*, entity_archetypes::init_bot};
use tracing::{trace, warn};

type SpawnSystemMut = (
    UnsafeView<EntityId, SpawnComponent>,
    UnsafeView<EntityId, SpawnQueueComponent>,
    UnsafeView<EntityId, EnergyComponent>,
    (
        UnsafeView<EntityId, SpawnBotComponent>,
        UnsafeView<EntityId, Bot>,
        UnsafeView<EntityId, HpComponent>,
        UnsafeView<EntityId, DecayComponent>,
        UnsafeView<EntityId, CarryComponent>,
        UnsafeView<EntityId, PositionComponent>,
        UnsafeView<EntityId, OwnedEntity>,
        UnsafeView<EntityId, EntityScript>,
    ),
);

pub fn update_spawns(
    (mut spawns, mut spawn_queue, mut energy, spawn_views): SpawnSystemMut,
    user_default_scripts: View<UserId, EntityScript>,
) {
    profile!("SpawnSystem update");

    let ss = spawns.iter_mut().filter(|(_, c)| c.spawning.is_none());
    let en = energy.iter_mut().filter(|(_, e)| e.energy == e.energy_max);
    let sq = spawn_queue.iter_mut();
    join!([ss, en, sq]).for_each(|(_spawn_id, (spawn, energy, queue))| {
        // spawns with 500 energy and no currently spawning bot
        if let Some(bot) = queue.queue.pop_back() {
            energy.energy -= 500;
            spawn.time_to_spawn = 10;
            spawn.spawning = Some(bot);
        }
    });

    spawns
        .iter_mut()
        .filter(|(_spawn_id, spawn_component)| spawn_component.spawning.is_some())
        .filter_map(|(spawn_id, spawn_component)| {
            spawn_component.time_to_spawn -= 1;
            if spawn_component.time_to_spawn == 0 {
                let bot = spawn_component.spawning.map(|b| (spawn_id, b));
                spawn_component.spawning = None;
                bot
            } else {
                None
            }
        })
        .for_each(|(spawn_id, entity_id)| {
            spawn_bot(spawn_id, entity_id, spawn_views, user_default_scripts)
        });
}

type SpawnBotMut = (
    UnsafeView<EntityId, SpawnBotComponent>,
    UnsafeView<EntityId, Bot>,
    UnsafeView<EntityId, HpComponent>,
    UnsafeView<EntityId, DecayComponent>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, PositionComponent>,
    UnsafeView<EntityId, OwnedEntity>,
    UnsafeView<EntityId, EntityScript>,
);

/// Spawns a bot from a spawn.
/// Removes the spawning bot from the spawn and initializes a bot in the world
fn spawn_bot(
    spawn_id: EntityId,
    entity_id: EntityId,
    (mut spawn_bots, bots, hps, decay, carry, positions, owned, script_table): SpawnBotMut,
    user_default_scripts: View<UserId, EntityScript>,
) {
    trace!(
        "spawn_bot spawn_id: {:?} entity_id: {:?}",
        spawn_id,
        entity_id
    );

    match spawn_bots.delete(entity_id) {
        Some(_) => (),
        None => {
            warn!("Spawning bot {:?} was not found", entity_id);
            return;
        }
    };

    let owner = owned
        .get_by_id(spawn_id)
        .map(|OwnedEntity { owner_id }| owner_id.0);

    let pos = positions
        .get_by_id(spawn_id)
        .copied()
        .expect("Spawn should have position")
        .0;

    init_bot(
        entity_id,
        owner,
        pos,
        (bots, hps, decay, carry, positions, owned, script_table),
        user_default_scripts,
    );

    trace!(
        "spawn_bot spawn_id: {:?} entity_id: {:?} - done",
        spawn_id,
        entity_id
    );
}
