#![feature(test)]
extern crate test;

#[macro_use]
extern crate log;

pub mod api;
pub mod model;
pub mod prelude;
pub mod storage;
pub mod tables;

mod intents;
mod systems;
mod utils;

use systems::execute_world_update;
use systems::intent_execution::execute_intents;
use systems::script_execution::execute_scripts;
use tables::{BTreeTable, MortonTable, VecTable};

pub fn forward(storage: &mut storage::Storage) -> Result<(), Box<dyn std::error::Error>> {
    profile!("forward world state");

    info!("Executing scripts");
    let final_intents = execute_scripts(storage);
    info!("Executing scripts - done");

    storage.signal_done(&final_intents);

    info!("Executing intents");
    execute_intents(final_intents, storage);
    info!("Executing intents - done");
    info!("Executing systems update");
    execute_world_update(storage);
    info!("Executing systems update - done");

    crate::utils::profiler::save_global();
    info!("-----------Tick {} done-----------", storage.time());
    Ok(())
}

pub fn init_inmemory_storage() -> storage::Storage {
    use model::components::*;

    profile!("init_inmemory_storage");
    debug!("Init Storage");

    let mut storage = storage::Storage::new();

    storage.add_entity_table::<Bot>(VecTable::new());
    storage.add_entity_table::<PositionComponent>(VecTable::new());
    storage.add_entity_table::<SpawnBotComponent>(BTreeTable::new());
    storage.add_entity_table::<DecayComponent>(BTreeTable::new());
    storage.add_entity_table::<CarryComponent>(BTreeTable::new());
    storage.add_entity_table::<Structure>(BTreeTable::new());
    storage.add_entity_table::<HpComponent>(BTreeTable::new());
    storage.add_entity_table::<EnergyRegenComponent>(BTreeTable::new());
    storage.add_entity_table::<EnergyComponent>(BTreeTable::new());
    storage.add_entity_table::<ResourceComponent>(BTreeTable::new());
    storage.add_entity_table::<DecayComponent>(BTreeTable::new());
    storage.add_entity_table::<EntityScript>(BTreeTable::new());
    storage.add_entity_table::<SpawnComponent>(BTreeTable::new());
    storage.add_entity_table::<OwnedEntity>(BTreeTable::new());

    storage.add_log_table::<LogEntry>(BTreeTable::new());

    storage.add_user_table::<UserComponent>(BTreeTable::new());

    storage.add_point_table::<TerrainComponent>(MortonTable::with_capacity(1024));
    storage.add_point_table::<EntityComponent>(MortonTable::with_capacity(1024));

    storage.add_scripts_table::<ScriptComponent>(BTreeTable::new());

    debug!("Init Storage done");
    storage
}
