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
        let energy_it = unsafe { energy.as_mut().iter_mut() };
        let join = JoinIterator::new(energy_it, energy_regen.iter());
        join.for_each(|(_id, (e, er))| {
            e.energy = (e.energy + er.amount).min(e.energy_max);
        });
    }
}
