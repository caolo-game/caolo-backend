pub mod intent_execution;
pub mod pathfinding;
pub mod script_execution;

use crate::model::{self, Circle, EntityId, Point};
use crate::profile;
use crate::storage::{views::*, Storage};
use crate::tables::{JoinIterator, Table};
use rand::Rng;

pub fn execute_world_update(storage: &mut Storage) {
    profile!("execute_world_update");

    update_energy(
        UnsafeView::from(storage as &mut _),
        View::from(storage as &_),
    );
    update_spawns(
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
    );
    update_decay(
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        storage,
    );
    update_minerals(
        UnsafeView::from(storage as &mut _),
        UnsafeView::from(storage as &mut _),
        View::from(storage as &_),
        View::from(storage as &_),
    );
    update_positions(
        UnsafeView::from(storage as &mut _),
        View::from(storage as &_),
    );
}

pub fn update_energy(
    mut energy: UnsafeView<EntityId, model::EnergyComponent>,
    energy_regen: View<EntityId, model::EnergyRegenComponent>,
) {
    let changeset = JoinIterator::new(energy.iter(), energy_regen.iter())
        .map(|(id, (e, er))| {
            let mut e = e.clone();
            e.energy = (e.energy + er.amount).min(e.energy_max);
            (id, e)
        })
        .collect::<Vec<_>>();
    for (id, e) in changeset.into_iter() {
        energy.insert_or_update(id, e);
    }
}

pub fn update_spawns(
    mut spawns: UnsafeView<EntityId, model::SpawnComponent>,
    spawn_bots: UnsafeView<EntityId, model::SpawnBotComponent>,
    bots: UnsafeView<EntityId, model::Bot>,
    hps: UnsafeView<EntityId, model::HpComponent>,
    decay: UnsafeView<EntityId, model::DecayComponent>,
    carry: UnsafeView<EntityId, model::CarryComponent>,
    positions: UnsafeView<EntityId, model::PositionComponent>,
    owned: UnsafeView<EntityId, model::OwnedEntity>,
) {
    let changeset = spawns
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
        spawns.insert_or_update(id, s);
        if let Some(e) = e {
            spawn_bot(id, e, spawn_bots, bots, hps, decay, carry, positions, owned);
        }
    }
}

/// Spawns a bot from a spawn.
/// Removes the spawning bot from the spawn and initializes a bot in the world
fn spawn_bot(
    spawn_id: model::EntityId,
    entity_id: model::EntityId,
    mut spawn_bots: UnsafeView<EntityId, model::SpawnBotComponent>,
    mut bots: UnsafeView<EntityId, model::Bot>,
    mut hps: UnsafeView<EntityId, model::HpComponent>,
    mut decay: UnsafeView<EntityId, model::DecayComponent>,
    mut carry: UnsafeView<EntityId, model::CarryComponent>,
    mut positions: UnsafeView<EntityId, model::PositionComponent>,
    mut owned: UnsafeView<EntityId, model::OwnedEntity>,
) {
    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?}",
        spawn_id, entity_id
    );

    let bot = spawn_bots
        .delete(&entity_id)
        .expect("Spawning bot was not found");
    bots.insert_or_update(entity_id, bot.bot);
    hps.insert_or_update(
        entity_id,
        crate::model::HpComponent {
            hp: 100,
            hp_max: 100,
        },
    );
    decay.insert_or_update(
        entity_id,
        crate::model::DecayComponent {
            eta: 20,
            t: 100,
            hp_amount: 100,
        },
    );
    carry.insert_or_update(
        entity_id,
        crate::model::CarryComponent {
            carry: 0,
            carry_max: 50,
        },
    );

    let pos = positions
        .get_by_id(&spawn_id)
        .cloned()
        .expect("Spawn should have position");
    positions.insert_or_update(entity_id, pos);

    let owner = owned.get_by_id(&spawn_id).cloned();
    if let Some(owner) = owner {
        owned.insert_or_update(entity_id, owner);
    }

    debug!(
        "spawn_bot spawn_id: {:?} entity_id: {:?} - done",
        spawn_id, entity_id
    );
}

pub fn update_decay(
    mut hps: UnsafeView<EntityId, model::HpComponent>,
    mut decays: UnsafeView<EntityId, model::DecayComponent>,
    storage: &mut Storage,
) {
    debug!("update decay system called");
    let changeset = JoinIterator::new(decays.iter(), hps.iter())
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
            hps.insert_or_update(id, hp);
            decays.insert_or_update(id, d);
        }
    }
    debug!("update decay system done");
}

pub fn update_minerals(
    mut entity_positions: UnsafeView<EntityId, model::PositionComponent>,
    mut energy: UnsafeView<EntityId, model::EnergyComponent>,
    position_entities: View<Point, model::EntityComponent>,
    resources: View<EntityId, model::ResourceComponent>,
) {
    debug!("update minerals system called");

    let mut rng = rand::thread_rng();

    let minerals = resources.iter().filter(|(_, r)| match r.0 {
        model::Resource::Mineral => true,
    });
    let changeset = JoinIterator::new(
        JoinIterator::new(minerals, entity_positions.iter()),
        energy.iter(),
    )
    .filter_map(|(id, ((_resource, position), energy))| {
        if energy.energy > 0 {
            return None;
        }

        let mut energy = energy.clone();
        let mut position = position.clone();

        energy.energy = energy.energy_max;

        position.0 = random_uncontested_pos_in_range(&*position_entities, &mut rng, -14, 15);

        Some((id, position, energy))
    })
    .collect::<Vec<_>>();

    for (id, pos, en) in changeset.into_iter() {
        debug!(
            "Mineral [{:?}] has been depleted, respawning at {:?}",
            id, pos
        );
        entity_positions.insert_or_update(id, pos);
        energy.insert_or_update(id, en);
    }

    debug!("update minerals system done");
}

fn random_uncontested_pos_in_range<T: crate::tables::PositionTable>(
    positions_table: &T,
    rng: &mut rand::rngs::ThreadRng,
    from: i32,
    to: i32,
) -> Point {
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
pub fn update_positions(
    mut position_entities: UnsafeView<Point, model::EntityComponent>,
    positions: View<EntityId, model::PositionComponent>,
) {
    let positions = positions
        .iter()
        .map(|(id, pos)| (pos.0, model::EntityComponent(id)))
        .collect::<Vec<_>>();

    position_entities.clear();

    for (point, entity) in positions.into_iter() {
        position_entities.insert(point, entity);
    }
}
