use super::*;
use crate::intents::check_spawn_intent;
use crate::model::{self, EntityId};
use crate::profile;
use crate::storage::Storage;
use crate::tables::StructureTable;
use caolo_api::structures::Spawn;
use rayon::prelude::*;

pub fn build_structure(
    id: EntityId,
    storage: &Storage,
) -> Option<caolo_api::structures::Structure> {
    let structure = storage.entity_table::<model::Structure>().get_by_id(&id)?;
    assemble_spawn(id, &structure, storage)
}

fn assemble_spawn(
    id: crate::model::EntityId,
    structure: &crate::model::Structure,
    storage: &Storage,
) -> Option<caolo_api::structures::Structure> {
    debug!("Assembling spawn {} {:?}", id, structure);
    storage
        .entity_table::<model::SpawnComponent>()
        .get_by_id(&id)
        .and_then(|spawn| {
            let hp = storage
                .entity_table::<model::HpComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Spawn should have hp");
                    None
                })?;
            let energy = storage
                .entity_table::<model::EnergyComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Spawn should have energy");
                    None
                })?;
            let position = storage
                .entity_table::<model::PositionComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Structures should have position");
                    None
                })?;
            let spawn = caolo_api::structures::Structure::Spawn(Spawn {
                id,
                owner_id: structure.owner_id,
                position: position.0,

                energy: energy.energy,
                energy_max: energy.energy_max,

                hp: hp.hp,
                hp_max: hp.hp_max,

                time_to_spawn: spawn.time_to_spawn,
                spawning: spawn.spawning,
            });
            Some(spawn)
        })
}
