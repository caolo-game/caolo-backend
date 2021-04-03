use crate::components;
use crate::indices::{EntityId, UserId};
use crate::scripting_api::OperationResult;
use crate::storage::views::View;
use crate::tables::traits::Table;
use serde::{Deserialize, Serialize};
use slog::{debug, Logger};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MineIntent {
    pub bot: EntityId,
    pub resource: EntityId,
}

type CheckInput<'a> = (
    View<'a, EntityId, components::Bot>,
    View<'a, EntityId, components::OwnedEntity>,
    View<'a, EntityId, components::PositionComponent>,
    View<'a, EntityId, components::ResourceComponent>,
    View<'a, EntityId, components::EnergyComponent>,
    View<'a, EntityId, components::CarryComponent>,
);

pub fn check_mine_intent(
    logger: &Logger,
    intent: &MineIntent,
    userid: UserId,
    (bots_table, owner_ids_table, positions_table, resources_table, energy_table, carry_table): CheckInput,
) -> OperationResult {
    let bot = intent.bot;
    match bots_table.get_by_id(bot) {
        Some(_) => {
            let owner_id = owner_ids_table.get_by_id(bot);
            if owner_id.map(|bot| bot.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let botpos = match positions_table.get_by_id(bot) {
        Some(pos) => pos,
        None => {
            debug!(logger, "Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    let target = intent.resource;
    let mineralpos = match positions_table.get_by_id(target) {
        Some(pos) => pos,
        None => {
            debug!(logger, "{:?} has no position", target);
            return OperationResult::InvalidInput;
        }
    };

    match carry_table.get_by_id(bot) {
        Some(carry) => {
            if carry.carry >= carry.carry_max {
                debug!(logger, "{:?} is full", bot);
                return OperationResult::Full;
            }
        }
        None => {
            debug!(logger, "{:?} has no carry component", bot);
            return OperationResult::InvalidInput;
        }
    }

    if botpos.0.room != mineralpos.0.room || botpos.0.pos.hex_distance(mineralpos.0.pos) > 1 {
        return OperationResult::NotInRange;
    }

    match resources_table.get_by_id(target) {
        Some(components::ResourceComponent(components::Resource::Energy)) => {
            match energy_table.get_by_id(target) {
                Some(energy) => {
                    if energy.energy > 0 {
                        OperationResult::Ok
                    } else {
                        OperationResult::Empty
                    }
                }
                None => {
                    debug!(logger, "Mineral has no energy component!");
                    OperationResult::InvalidInput
                }
            }
        }
        Some(_) | None => {
            debug!(logger, "{:?} is not a resource!", target);
            OperationResult::InvalidInput
        }
    }
}
