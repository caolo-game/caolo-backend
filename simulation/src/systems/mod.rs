pub mod execution;
pub mod pathfinding;

use crate::model::{self, bots::spawn_bot};
use crate::profile;
use crate::storage::Storage;
use crate::tables::JoinIterator;
use rand::Rng;

pub fn execute_world_update(storage: &mut Storage) {
    profile!("execute_world_update");

    update_energy(storage);
    update_spawns(storage);
    update_decay(storage);
    update_minerals(storage);
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
        energy.insert(id, e);
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
            .insert(id, s);
        if let Some(e) = e {
            spawn_bot(id, e, storage);
        }
    }
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
                .insert(id, hp);
            storage
                .entity_table_mut::<model::DecayComponent>()
                .insert(id, d);
        }
    }
    debug!("update decay system done");
}

pub fn update_minerals(storage: &mut Storage) {
    debug!("update minerals system called");

    let positions = storage.entity_table::<model::PositionComponent>();
    let energy = storage.entity_table::<model::EnergyComponent>();
    let resources = storage.entity_table::<model::Resource>();

    let mut rng = rand::thread_rng();

    let changeset = JoinIterator::new(
        JoinIterator::new(resources.iter(), positions.iter()),
        energy.iter(),
    )
    .filter_map(|(id, ((resource, position), energy))| match resource {
        model::Resource::Mineral => {
            if energy.energy > 0 {
                return None;
            }

            let mut energy = energy.clone();
            let mut position = position.clone();

            energy.energy = energy.energy_max;

            position.0 = random_uncontested_pos_in_range(positions, &mut rng, -14, 15);

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
            .insert(id, pos);
        storage
            .entity_table_mut::<model::EnergyComponent>()
            .insert(id, en);
    }

    debug!("update minerals system done");
}

fn random_uncontested_pos_in_range(
    positions_table: &dyn crate::tables::PositionTable,
    rng: &mut rand::rngs::ThreadRng,
    from: i32,
    to: i32,
) -> model::Point {
    let mut pos = model::Point::default();
    loop {
        pos.x = rng.gen_range(from, to);
        pos.y = rng.gen_range(from, to);

        if positions_table.count_entities_in_range(&model::Circle {
            center: pos,
            radius: 0,
        }) == 0
        {
            break;
        }
    }
    pos
}
