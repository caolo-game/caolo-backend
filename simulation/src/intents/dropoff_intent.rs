use super::*;
use crate::model::{self, EntityId, ResourceType};
use crate::prelude::*;
use caolo_api::OperationResult;

pub const DROPOFF_RANGE: u64 = 1;

#[derive(Debug, Clone)]
pub struct DropoffIntent {
    pub bot: EntityId,
    pub structure: EntityId,
    pub amount: u16,
    pub ty: ResourceType,
}

impl DropoffIntent {
    pub fn execute(&self, storage: &mut crate::storage::Storage) -> IntentResult {
        // dropoff amount = min(bot carry , amount , structure capacity)
        let mut carry_component = storage
            .entity_table::<model::CarryComponent>()
            .get_by_id(&self.bot)
            .cloned()
            .ok_or_else(|| "Bot has no carry")?;
        let mut store_component = storage
            .entity_table::<model::EnergyComponent>()
            .get_by_id(&self.bot)
            .cloned()
            .ok_or_else(|| "Bot has no carry")?;
        let dropoff = self
            .amount
            .min(carry_component.carry)
            .min(store_component.energy_max - store_component.energy);

        store_component.energy += dropoff;
        carry_component.carry -= dropoff;

        storage
            .entity_table_mut::<model::CarryComponent>()
            .insert(self.bot, carry_component);
        storage
            .entity_table_mut::<model::EnergyComponent>()
            .insert(self.structure, store_component);

        Ok(())
    }
}

/// A valid dropoff intent has the following characteristics:
/// - the bot is the user's
/// - the bot is carrying resource of type ty
/// - the target is not full
/// - the target is within dropoff range
pub fn check_dropoff_intent(
    intent: &caolo_api::bots::DropoffIntent,
    userid: model::UserId,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let id = EntityId(intent.id);
    match storage.entity_table::<model::Bot>().get_by_id(&id) {
        Some(_) => {
            let owner_id = storage.entity_table::<model::OwnedEntity>().get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    if let Some(carry) = storage
        .entity_table::<model::CarryComponent>()
        .get_by_id(&id)
    {
        if carry.carry == 0 {
            return OperationResult::Empty;
        }
    } else {
        return OperationResult::Empty;
    }

    let positions = storage.entity_table::<model::PositionComponent>();

    let target = EntityId(intent.target);
    let nearby = positions.get_by_id(&id).and_then(|botpos| {
        positions
            .get_by_id(&target)
            .map(|targetpos| targetpos.0.hex_distance(botpos.0) <= DROPOFF_RANGE)
    });
    match nearby {
        None => {
            error!("Bot or target has no position components {:?}", intent);
            return OperationResult::InvalidInput;
        }
        Some(false) => {
            return OperationResult::NotInRange;
        }
        Some(true) => {
            let capacity = storage
                .entity_table::<model::EnergyComponent>()
                .get_by_id(&target);
            if capacity.is_none() {
                error!("Target has no energy component {:?}", intent);
                return OperationResult::InvalidInput;
            }
            let capacity = capacity.unwrap();
            if capacity.energy < capacity.energy_max {
                return OperationResult::Ok;
            }
            return OperationResult::Full;
        }
    }
}
