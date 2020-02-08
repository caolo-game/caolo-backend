use crate::protos::world::Bot as BotMsg;

use crate::protos::world::LogEntry as LogMsg;
use caolo_sim::model::{
    components::{Bot, LogEntry, OwnedEntity, PositionComponent},
    indices::EntityTime,
    EntityId,
};
use caolo_sim::storage::views::View;
use caolo_sim::tables::JoinIterator;

pub fn build_bots<'a>(
    (bots, positions, owned_entities): (
        View<'a, EntityId, Bot>,
        View<'a, EntityId, PositionComponent>,
        View<'a, EntityId, OwnedEntity>,
    ),
) -> impl Iterator<Item = BotMsg> + 'a {
    let bots = bots.reborrow().iter();
    let positions = positions.reborrow().iter();
    JoinIterator::new(bots, positions).map(move |(id, (_bot, pos))| {
        let mut msg = BotMsg::default();
        msg.set_id(id.0);
        msg.mut_position().set_q(pos.0.x);
        msg.mut_position().set_r(pos.0.y);
        msg.mut_owner().clear();
        if let Some(owner) = owned_entities.get_by_id(&id) {
            *msg.mut_owner() = owner.owner_id.0.as_bytes().to_vec();
        }
        msg
    })
}

pub fn build_logs<'a>(v: View<'a, EntityTime, LogEntry>) -> impl Iterator<Item = LogMsg> + 'a {
    v.reborrow()
        .iter()
        .map(|(EntityTime(EntityId(eid), time), entries)| {
            let mut msg = LogMsg::new();
            msg.set_entity_id(eid);
            msg.set_time(time);
            for e in entries.payload.iter() {
                msg.mut_payload().push(e.clone());
            }
            msg
        })
}
