pub mod execution;
pub mod pathfinding;

use crate::model::{self, Circle};
use crate::profile;
use crate::storage::Storage;
use crate::tables::{JoinIterator, Table};
use rand::Rng;

pub fn execute_world_update(storage: &mut Storage) {
    profile!("execute_world_update");

    update_energy(storage);
    update_spawns(storage);
    update_decay(storage);
    update_minerals(storage);

    update_positions(storage);
}

pub fn update_energy(storage: &mut Storage) {
    let energy_regen = storage.entity_table::<model::EnergyRegenComponent>();
    let energy = storage.entity_table::<model::EnergyComponent>();
    let changeset = JoinIterator::new(energy.iter(), energy_regen.iter())
        .map(|(id, (e, er))| {
            let mut e = e.clone();
            e.energy = (e.energy + er.amount).min(e.energy_max);
            (id, e)
        })
        .collect::<Vec<_>>();
    let energy = storage.entity_table_mut::<model::EnergyComponent>();
    for (id, e) in changeset.into_iter() {
        energy.insert_or_update(id, e);
    }
}

pub fn update_spawns(storage: &mut Storage) {
    let spawn = storage.entity_table::<model::SpawnComponent>();

    let changeset = spawn
        .iter()
        .map(|(id, s)| {
            let mut s = s.clone();
            if s.time_to_spawn > 0 {
                s.time_to_spawn -= 1;
            }
            let mut bot = None;
            if s.spawning.is_some() && s.time_to_spawn == 0 {
                bot = s.spawning;
                s.spawning = None;
            }
            (id, s, bot)
        })
        .collect::<Vec<_>>();

    for (id, s, e) in changeset.into_iter() {
        storage
            .entity_table_mut::<model::SpawnComponent>()
            .insert_or_update(id, s);
        if let Some(e) = e {
            spawn_bot(id, e, storage);
        }
    }
}

/// Spawns a bot from a spawn.
/// Removes the spawning bot from the spawn and initializes a bot in the world
pub fn spawn_bot(spawn_id: model::EntityId, entity_id: model::EntityId, storage: &mut Storage) {
    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?}",
        spawn_id, entity_id
    );

    let bot = storage
        .entity_table_mut::<model::SpawnBotComponent>()
        .delete(&entity_id)
        .expect("Spawning bot was not found");
    storage
        .entity_table_mut::<model::Bot>()
        .insert_or_update(entity_id, bot.bot);
    storage
        .entity_table_mut::<model::HpComponent>()
        .insert_or_update(
            entity_id,
            crate::model::HpComponent {
                hp: 100,
                hp_max: 100,
            },
        );
    storage
        .entity_table_mut::<model::DecayComponent>()
        .insert_or_update(
            entity_id,
            crate::model::DecayComponent {
                eta: 20,
                t: 100,
                hp_amount: 100,
            },
        );
    storage
        .entity_table_mut::<model::CarryComponent>()
        .insert_or_update(
            entity_id,
            crate::model::CarryComponent {
                carry: 0,
                carry_max: 50,
            },
        );

    let positions = storage.entity_table_mut::<model::PositionComponent>();
    let pos = positions
        .get_by_id(&spawn_id)
        .cloned()
        .expect("Spawn should have position");
    positions.insert_or_update(entity_id, pos);

    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?} - done",
        spawn_id, entity_id
    );
}

pub fn update_decay(storage: &mut Storage) {
    debug!("update decay system called");
    let decay = storage.entity_table::<model::DecayComponent>();
    let hp = storage.entity_table::<model::HpComponent>();
    let changeset = JoinIterator::new(decay.iter(), hp.iter())
        .map(|(id, (d, hp))| {
            let mut d = d.clone();
            let mut hp = hp.clone();
            if d.t > 0 {
                d.t -= 1;
            }
            if d.t == 0 {
                hp.hp -= hp.hp.min(d.hp_amount);
            }
            (id, d, hp)
        })
        .collect::<Vec<_>>();

    for (id, d, hp) in changeset.into_iter() {
        if hp.hp == 0 {
            debug!("Entity {:?} has died, deleting", id);
            storage.delete_entity(id);
        } else {
            storage
                .entity_table_mut::<model::HpComponent>()
                .insert_or_update(id, hp);
            storage
                .entity_table_mut::<model::DecayComponent>()
                .insert_or_update(id, d);
        }
    }
    debug!("update decay system done");
}

pub fn update_minerals(storage: &mut Storage) {
    debug!("update minerals system called");

    let entity_positions = storage.entity_table::<model::PositionComponent>();
    let position_entities = storage.point_table::<model::EntityComponent>();
    let energy = storage.entity_table::<model::EnergyComponent>();
    let resources = storage.entity_table::<model::ResourceComponent>();

    let mut rng = rand::thread_rng();

    let changeset = JoinIterator::new(
        JoinIterator::new(resources.iter(), entity_positions.iter()),
        energy.iter(),
    )
    .filter_map(|(id, ((resource, position), energy))| match resource.0 {
        model::Resource::Mineral => {
            if energy.energy > 0 {
                return None;
            }

            let mut energy = energy.clone();
            let mut position = position.clone();

            energy.energy = energy.energy_max;

            position.0 = random_uncontested_pos_in_range(position_entities, &mut rng, -14, 15);

            Some((id, position, energy))
        }
    })
    .collect::<Vec<_>>();

    for (id, pos, en) in changeset.into_iter() {
        debug!(
            "Mineral [{:?}] has been depleted, respawning at {:?}",
            id, pos
        );
        storage
            .entity_table_mut::<model::PositionComponent>()
            .insert_or_update(id, pos);
        storage
            .entity_table_mut::<model::EnergyComponent>()
            .insert_or_update(id, en);
    }

    debug!("update minerals system done");
}

fn random_uncontested_pos_in_range<T: crate::tables::PositionTable>(
    positions_table: &T,
    rng: &mut rand::rngs::ThreadRng,
    from: i32,
    to: i32,
) -> model::Point {
    let mut pos = model::Point::default();
    loop {
        pos.x = rng.gen_range(from, to);
        pos.y = rng.gen_range(from, to);

        let circle = Circle {
            center: pos,
            radius: 1,
        };
        if positions_table.count_entities_in_range(&circle) == 0 {
            break;
        }
    }
    pos
}

/// Rebuild the point tables
pub fn update_positions(storage: &mut Storage) {
    use model::EntityComponent;
    use model::PositionComponent;

    let positions = storage.entity_table::<PositionComponent>();
    let positions = positions
        .iter()
        .map(|(id, pos)| (pos.0, EntityComponent(id)))
        .collect::<Vec<_>>();

    let position_entities = storage.point_table_mut::<EntityComponent>();
    position_entities.clear();

    for (point, entity) in positions.into_iter() {
        position_entities.insert(point, entity);
    }
}
