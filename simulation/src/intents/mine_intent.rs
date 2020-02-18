use crate::model::{bots, components, EntityId, OperationResult, UserId};
use crate::storage::views::View;

#[derive(Debug, Clone)]
pub struct MineIntent {
    pub bot: EntityId,
    pub resource: EntityId,
}

pub fn check_mine_intent(
    intent: &bots::MineIntent,
    userid: UserId,
    bots: View<EntityId, components::Bot>,
    owner_ids: View<EntityId, components::OwnedEntity>,
    positions: View<EntityId, components::PositionComponent>,
    resources: View<EntityId, components::ResourceComponent>,
    energy: View<EntityId, components::EnergyComponent>,
) -> OperationResult {
    let id = intent.id;
    match bots.get_by_id(&id) {
        Some(_) => {
            let owner_id = owner_ids.get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let botpos = match positions.get_by_id(&id) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    let target = intent.target;
    let mineralpos = match positions.get_by_id(&target) {
        Some(pos) => pos,
        None => {
            debug!("Mineral has no position");
            return OperationResult::InvalidInput;
        }
    };

    if botpos.0.hex_distance(mineralpos.0) > 1 {
        return OperationResult::NotInRange;
    }

    match resources.get_by_id(&target) {
        Some(components::ResourceComponent(components::Resource::Mineral)) => {
            match energy.get_by_id(&target) {
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
