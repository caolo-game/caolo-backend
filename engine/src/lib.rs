#[macro_use]
extern crate log;

pub mod api;
pub mod model;
pub mod storage;
pub mod tables;

mod intents;
mod systems;
mod utils;

use caolo_api::EntityId;
use systems::execute_world_update;
use systems::execution::{execute_intents, execute_scripts};

pub fn forward(storage: &mut storage::Storage) -> Result<(), Box<dyn std::error::Error>> {
    compile_scripts(storage);
    let final_intents = execute_scripts(storage);

    storage.signal_done(&final_intents);

    info!("Executing intents {}", final_intents.len());
    execute_intents(final_intents, storage);
    info!("Executing intents - done");
    info!("Executing systems update");
    execute_world_update(storage);
    info!("Executing systems update - done");

    Ok(())
}

fn compile_scripts(storage: &mut storage::Storage) {
    use caolo_api::{Script, ScriptId};
    use rayon::prelude::*;

    info!("Compiling scripts");

    let scripts = storage.scripts_table::<Script>();
    let to_compile: Vec<(ScriptId, Script)> = scripts
        .iter()
        .filter(|(_, s)| s.compiled.is_none())
        .map(|(id, s)| (id, s.clone()))
        .collect();
    let changeset: Vec<(ScriptId, Script)> = to_compile
        .into_par_iter()
        .filter_map(|(id, mut script)| {
            let compiled = cao_lang::prelude::Compiler::compile(script.script.clone())
                .map_err(|e| {
                    error!("Script {:?} failed to compile {:?}", id, e);
                })
                .ok()?;
            script.compiled = Some(compiled);
            Some((id, script))
        })
        .collect();
    for (id, script) in changeset.into_iter() {
        storage
            .scripts_table_mut::<caolo_api::Script>()
            .insert(id, script);
    }

    info!("Compiling scripts done");
}

pub fn init_inmemory_storage() -> storage::Storage {
    use model::*;
    use tables::*;

    debug!("Init InMemoryStorage");

    let mut storage = storage::Storage::new();

    storage.add_entity_table::<Bot>(Table::default_inmemory());
    storage.add_entity_table::<SpawnBotComponent>(Table::default_inmemory());
    storage.add_entity_table::<DecayComponent>(Table::default_inmemory());
    storage.add_entity_table::<CarryComponent>(Table::default_inmemory());
    storage.add_entity_table::<Structure>(Table::default_inmemory());
    storage.add_entity_table::<HpComponent>(Table::default_inmemory());
    storage.add_entity_table::<EnergyRegenComponent>(Table::default_inmemory());
    storage.add_entity_table::<EnergyComponent>(Table::default_inmemory());
    storage.add_entity_table::<PositionComponent>(Table::default_inmemory());
    storage.add_entity_table::<ResourceType>(Table::default_inmemory());
    storage.add_entity_table::<DecayComponent>(Table::default_inmemory());
    storage.add_entity_table::<EntityScript>(Table::default_inmemory());
    storage.add_entity_table::<SpawnComponent>(Table::default_inmemory());
    storage.add_entity_table::<Resource>(Table::default_inmemory());

    storage.add_log_table::<LogEntry>(Table::default_inmemory());

    storage.add_user_table::<UserData>(Table::default_inmemory());

    storage.add_point_table::<TileTerrainType>(Table::default_inmemory());

    storage.add_scripts_table::<caolo_api::Script>(Table::default_inmemory());

    debug!("Init InMemoryStorage done");
    storage
}

#[allow(unused)]
fn uncontested_pos(
    positions_table: &dyn tables::PositionTable,
    rng: &mut rand::rngs::ThreadRng,
) -> caolo_api::point::Point {
    use caolo_api::point::{Circle, Point};
    use rand::Rng;

    let mut pos = Point::default();
    loop {
        pos.x = rng.gen_range(-19, 20);
        pos.y = rng.gen_range(-19, 20);

        if positions_table.count_entities_in_range(&Circle {
            center: pos,
            radius: 0,
        }) == 0
        {
            break;
        }
    }
    pos
}
