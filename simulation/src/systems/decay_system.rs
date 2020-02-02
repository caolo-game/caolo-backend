use super::System;
use crate::model::{self, EntityId};
use crate::storage::views::{DeleteEntityView, UnsafeView};
use crate::tables::JoinIterator;

pub struct DecaySystem;

impl<'a> System<'a> for DecaySystem {
    type Mut = (
        UnsafeView<EntityId, model::HpComponent>,
        UnsafeView<EntityId, model::DecayComponent>,
        DeleteEntityView,
    );
    type Const = ();

    fn update(&mut self, (mut hps, mut decays, mut delete): Self::Mut, _: Self::Const) {
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
                unsafe {
                    delete.delete_entity(id);
                }
            } else {
                unsafe {
                    hps.as_mut().insert_or_update(id, hp);
                    decays.as_mut().insert_or_update(id, d);
                }
            }
        }
        debug!("update decay system done");
    }
}
