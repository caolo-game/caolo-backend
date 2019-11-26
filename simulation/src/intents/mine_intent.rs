use super::*;
use crate::model::{self, EntityId};
use caolo_api::OperationResult;

const MINE_AMOUNT: u16 = 10; // TODO: get from bot body

#[derive(Debug, Clone)]
pub struct MineIntent {
    pub bot: EntityId,
    pub resource: EntityId,
}

impl MineIntent {
    pub fn execute(&self, storage: &mut Storage) -> IntentResult {
        debug!("Bot [{:?}] is mining [{:?}]", self.bot, self.resource);
        match storage
            .entity_table::<model::Resource>()
            .get_by_id(&self.resource)
        {
            None => Err("Resource not found".into()),
            Some(crate::model::Resource::Mineral) => {
                let mut energy = match storage
                    .entity_table::<model::EnergyComponent>()
                    .get_by_id(&self.resource)
                {
                    Some(energy) => {
                        if energy.energy == 0 {
                            return Err("Mineral is empty!".into());
                        }
                        energy
                    }
                    None => {
                        return Err("Mineral has no energy component!".into());
                    }
                };
                let mut carry = storage
                    .entity_table::<model::CarryComponent>()
                    .get_by_id(&self.bot)
                    .ok_or_else(|| {
                        error!("MineIntent bot has no carry component");
                        "Bot has no carry"
                    })?;
                let mined = energy.energy.min(MINE_AMOUNT); // Max amount that can be mined
                let mined = (carry.carry_max - carry.carry).min(mined); // Max amount the bot can carry

                carry.carry += mined;
                energy.energy -= mined;

                storage
                    .entity_table_mut::<model::CarryComponent>()
                    .insert(self.bot, carry);
                storage
                    .entity_table_mut::<model::EnergyComponent>()
                    .insert(self.resource, energy);
                debug!("Mine succeeded");
                Ok(())
            }
        }
    }
}

pub fn check_mine_intent(
    intent: &caolo_api::bots::MineIntent,
    userid: caolo_api::UserId,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let bots = storage.entity_table::<model::Bot>();

    match bots.get_by_id(&intent.id) {
        Some(_) => {
            let owner_id = storage
                .entity_table::<model::OwnedEntity>()
                .get_by_id(&intent.id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let positions = storage.entity_table::<model::PositionComponent>();

    let botpos = match positions.get_by_id(&intent.id) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    let mineralpos = match positions.get_by_id(&intent.target) {
        Some(pos) => pos,
        None => {
            debug!("Mineral has no position");
            return OperationResult::InvalidInput;
        }
    };

    if botpos.0.hex_distance(mineralpos.0) > 1 {
        return OperationResult::NotInRange;
    }

    match storage
        .entity_table::<model::Resource>()
        .get_by_id(&intent.target)
    {
        Some(model::Resource::Mineral) => {
            match storage
                .entity_table::<model::EnergyComponent>()
                .get_by_id(&intent.target)
            {
                Some(energy) => {
                    if energy.energy > 0 {
                        OperationResult::Ok
                    } else {
                        OperationResult::Empty
                    }
                }
                None => {
                    debug!("Mineral has no energy component!");
                    OperationResult::InvalidInput
                }
            }
        }
        None => {
            debug!("Target is not a resource!");
            OperationResult::InvalidInput
        }
    }
}
