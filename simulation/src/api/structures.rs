use crate::model::{self, EntityId};
use crate::storage::{views::View, Storage};
use caolo_api::structures::Spawn;

pub fn build_structure(
    id: EntityId,
    storage: &Storage,
) -> Option<caolo_api::structures::Structure> {
    let structure = storage.entity_table::<model::Structure>().get_by_id(&id)?;
    assemble_spawn(
        id,
        &structure,
        View::from(storage),
        View::from(storage),
        View::from(storage),
        View::from(storage),
        View::from(storage),
    )
}

fn assemble_spawn(
    id: model::EntityId,
    structure: &model::Structure,
    spawns: View<EntityId, model::SpawnComponent>,
    positions: View<EntityId, model::PositionComponent>,
    hps: View<EntityId, model::HpComponent>,
    energys: View<EntityId, model::EnergyComponent>,
    owner_ids: View<EntityId, model::OwnedEntity>,
) -> Option<caolo_api::structures::Structure> {
    debug!("Assembling spawn {:?} {:?}", id, structure);
    spawns.get_by_id(&id).and_then(|spawn| {
        let position = positions.get_by_id(&id).or_else(|| {
            error!("Structures should have position");
            None
        })?;
        let hp = hps.get_by_id(&id).or_else(|| {
            error!("Spawn should have hp");
            None
        })?;
        let energy = energys.get_by_id(&id).or_else(|| {
            error!("Spawn should have energy");
            None
        })?;
        let owner_id = owner_ids.get_by_id(&id);
        let spawn = caolo_api::structures::Structure::Spawn(Spawn {
            id: id.0,
            owner_id: owner_id.map(|id| id.owner_id.0),
            position: position.0,

            energy: energy.energy,
            energy_max: energy.energy_max,

            hp: hp.hp,
            hp_max: hp.hp_max,

            time_to_spawn: spawn.time_to_spawn,
            spawning: spawn.spawning.map(|x| x.0),
        });
        Some(spawn)
    })
}
