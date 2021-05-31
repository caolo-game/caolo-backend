pub mod attack_system;
pub mod death_system;
pub mod decay_system;
pub mod dropoff_intent_system;
pub mod energy_system;
pub mod log_intent_system;
pub mod log_system;
pub mod mine_intent_system;
pub mod mineral_system;
pub mod move_intent_system;
pub mod path_cache_intent_system;
pub mod positions_system;
pub mod say_intent_system;
pub mod script_execution;
pub mod script_history_system;
pub mod spawn_system;

use attack_system::attack_system_update;
use death_system::death_update;
use decay_system::decay_update;
use dropoff_intent_system::dropoff_intents_update;
use energy_system::energy_update;
use log_intent_system::log_intents_update;
use log_system::log_update;
use mine_intent_system::mine_intents_update;
use mineral_system::mineral_update;
use move_intent_system::move_intents_update;
use path_cache_intent_system::path_cache_intents_update;
use positions_system::positions_update;
use say_intent_system::say_intents_update;
use script_history_system::script_history_update;
use spawn_system::{update_spawn_intents, update_spawns};

use crate::storage::views::{FromWorld, FromWorldMut};
use crate::{prelude::World, profile};

pub fn execute_world_update(storage: &mut World) {
    profile!("execute_systems_update");

    execute_intents(storage);
    execute_automated_systems(storage);
}

fn execute_intents(storage: &mut World) {
    profile!("execute_intents");

    // pre processing
    execute_update(spawn_system::update_cont_spawns, storage);

    // main processing
    execute_update(attack_system_update, storage);
    execute_update(move_intents_update, storage);
    execute_update(mine_intents_update, storage);
    execute_update(dropoff_intents_update, storage);
    execute_update(update_spawn_intents, storage);
    execute_update(log_intents_update, storage);
    execute_update(path_cache_intents_update, storage);
    execute_update(script_history_update, storage);
    execute_update(say_intents_update, storage);
}

/// Execute systems that run regardless of player actions
fn execute_automated_systems(storage: &mut World) {
    profile!("execute_automated_systems");

    execute_update(decay_update, storage);
    execute_update(death_update, storage);
    execute_update(energy_update, storage);
    execute_update(update_spawns, storage);
    execute_update(mineral_update, storage);
    execute_update(positions_update, storage);
    execute_update(log_update, storage);
}

#[inline]
fn execute_update<'a, M, C, Sys>(sys: Sys, storage: &'a mut World)
where
    Sys: Fn(M, C) + 'a,
    M: FromWorldMut + Clone + 'a,
    C: FromWorld<'a> + 'a,
{
    let m = M::from_world_mut(storage);
    let c = C::from_world(storage as &_);
    sys(M::clone(&m), c);
}
