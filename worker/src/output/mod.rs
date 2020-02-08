use crate::protos::world::Bot as BotMsg;
use caolo_sim::model::{
    components::{Bot, OwnedEntity, PositionComponent},
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
        msg.mut_position().set_x(pos.0.x);
        msg.mut_position().set_y(pos.0.y);
        msg.mut_owner().clear();
        if let Some(owner) = owned_entities.get_by_id(&id) {
            *msg.mut_owner() = owner.owner_id.0.as_bytes().to_vec();
        }
        msg
    })
}
