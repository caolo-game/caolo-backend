pub mod energy_system;
pub mod intent_execution;
pub mod mineral_system;
pub mod pathfinding;
pub mod positions_system;
pub mod script_execution;
pub mod spawn_system;

use crate::model::{self, EntityId};
use crate::profile;
use crate::storage::{
    views::{HasNew, HasNewMut, UnsafeView, View},
    Storage,
};
use crate::tables::JoinIterator;

pub trait System<'a> {
    // Requiring these traits instead of From impl disallows Storage as an `update` parameter
    // Thus requiring callers to explicitly state their dependencies
    type Mut: HasNewMut;
    type Const: HasNew<'a>;

    fn update(&mut self, m: Self::Mut, c: Self::Const);
}

pub fn execute_world_update(storage: &mut Storage) {
    profile!("execute_world_update");

    let mut energy_sys = energy_system::EnergySystem;
    energy_sys.update(
        UnsafeView::from(storage as &mut _),
        View::from(storage as &_),
    );

    let mut spawn_sys = spawn_system::SpawnSystem;
    spawn_sys.update(From::from(storage as &mut _), From::from(storage as &_));

    update_decay(From::from(storage as &mut _), storage);

    let mut mineral_sys = mineral_system::MineralSystem;
    mineral_sys.update(From::from(storage as &mut _), From::from(storage as &_));

    let mut positions_sys = positions_system::PositionSystem;
    positions_sys.update(From::from(storage as &mut _), From::from(storage as &_));
}

fn update_decay(
    (mut hps, mut decays): (
        UnsafeView<EntityId, model::HpComponent>,
        UnsafeView<EntityId, model::DecayComponent>,
    ),
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
            unsafe {
                hps.as_mut().insert_or_update(id, hp);
                decays.as_mut().insert_or_update(id, d);
            }
        }
    }
    debug!("update decay system done");
}
