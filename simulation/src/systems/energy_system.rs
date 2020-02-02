use super::System;
use crate::model::{self, EntityId};
use crate::storage::views::{UnsafeView, View};
use crate::tables::JoinIterator;

pub struct EnergySystem;

impl<'a> System<'a> for EnergySystem {
    type Mut = UnsafeView<EntityId, model::EnergyComponent>;
    type Const = View<'a, EntityId, model::EnergyRegenComponent>;

    fn update(
        &mut self,
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
            unsafe { energy.as_mut() }.insert_or_update(id, e);
        }
    }
}
