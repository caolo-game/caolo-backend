use super::*;
use crate::intents::{check_dropoff_intent, check_mine_intent, check_move_intent};
use crate::profile;
use crate::model::{self, EntityId};
use crate::storage::Storage;
use crate::tables::BotTable;
use rayon::prelude::*;

pub fn build_bot(id: EntityId, storage: &Storage) -> Option<caolo_api::bots::Bot> {
    let pos = storage
        .entity_table::<model::PositionComponent>()
        .get_by_id(&id)
        .or_else(|| {
            debug!("Bot {:?} could not be build because it has no position", id);
            None
        })?;

    let carry = storage
        .entity_table::<model::CarryComponent>()
        .get_by_id(&id)
        .unwrap_or_else(|| model::CarryComponent {
            carry: 0,
            carry_max: 0,
        });

    let bot = storage.entity_table::<model::Bot>().get_by_id(&id);

    bot.map(|bot| caolo_api::bots::Bot {
        id,
        speed: bot.speed,
        owner_id: bot.owner_id,
        position: pos.0,
        carry: carry.carry,
        carry_max: carry.carry_max,
    })
    .or_else(|| {
        debug!(
            "Bot {:?} could not be build because it has no bot component",
            id
        );
        None
    })
}
