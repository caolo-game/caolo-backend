use super::IntentExecutionSystem;
use crate::intents::MineIntent;
use crate::model::{
    components::{CarryComponent, EnergyComponent, Resource, ResourceComponent},
    EntityId,
};
use crate::storage::views::{UnsafeView, View};

pub const MINE_AMOUNT: u16 = 10; // TODO: get from bot body

pub struct MineSystem;

impl<'a> IntentExecutionSystem<'a> for MineSystem {
    type Mut = (
        UnsafeView<EntityId, EnergyComponent>,
        UnsafeView<EntityId, CarryComponent>,
    );
    type Const = (View<'a, EntityId, ResourceComponent>,);
    type Intent = MineIntent;

    fn execute(
        &mut self,
        (mut energy_table, mut carry_table): Self::Mut,
        (resource_table,): Self::Const,
        intents: &[Self::Intent],
    ) {
        for intent in intents {
            debug!("Bot [{:?}] is mining [{:?}]", intent.bot, intent.resource);
            match resource_table.get_by_id(&intent.resource) {
                None => warn!("Resource not found"),
                Some(ResourceComponent(Resource::Energy)) => {
                    let mut energy = match energy_table.get_by_id(&intent.resource) {
                        Some(energy) => {
                            if energy.energy == 0 {
                                debug!("Mineral is empty!");
                                continue;
                            }
                            energy.clone()
                        }
                        None => {
                            warn!("Mineral has no energy component!");
                            continue;
                        }
                    };
                    let mut carry = match carry_table.get_by_id(&intent.bot).cloned() {
                        Some(x) => x,
                        None => {
                            warn!("MineIntent bot has no carry component");
                            continue;
                        }
                    };
                    let mined = energy.energy.min(MINE_AMOUNT); // Max amount that can be mined
                    let mined = (carry.carry_max - carry.carry).min(mined); // Max amount the bot can carry

                    carry.carry += mined;
                    energy.energy -= mined;

                    unsafe {
                        carry_table.as_mut().insert_or_update(intent.bot, carry);
                        energy_table
                            .as_mut()
                            .insert_or_update(intent.resource, energy);
                    }
                    debug!("Mine succeeded");
                }
            }
        }
    }
}
