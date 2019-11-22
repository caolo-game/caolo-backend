use super::*;
use crate::model::{self, PositionComponent};
use crate::tables::PositionTable;
use caolo_api::OperationResult;

#[derive(Debug, Clone)]
pub struct MoveIntent {
    pub bot: EntityId,
    pub position: Point,
}

impl MoveIntent {
    pub fn execute(&self, storage: &mut Storage) -> IntentResult {
        debug!("Moving bot[{:?}] to {:?}", self.bot, self.position);

        let table = storage.entity_table::<PositionComponent>();

        if storage
            .entity_table::<model::Bot>()
            .get_by_id(&self.bot)
            .is_none()
        {
            debug!("Bot by id {:?} does not exist", self.bot);
            return Err("Bot not found".into());
        }

        let pos = PositionComponent(self.position);
        if 0 < table.count_entities_in_range(&Circle {
            center: pos.0,
            radius: 0,
        }) {
            debug!("Occupied");
            return Err("Occupied".into());
        }

        let table = storage.entity_table_mut::<PositionComponent>();

        table.insert(self.bot, pos);

        debug!("Move successful");

        Ok(())
    }
}

pub fn check_move_intent(
    intent: &caolo_api::bots::MoveIntent,
    userid: caolo_api::UserId,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let bots = storage.entity_table::<model::Bot>();
    let terrain = storage.point_table::<model::TileTerrainType>();

    let bot = match bots.get_by_id(&intent.id) {
        Some(bot) => {
            if bot.owner_id.map(|id| id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
            bot
        }
        None => return OperationResult::InvalidInput,
    };

    let pos = match storage
        .entity_table::<PositionComponent>()
        .get_by_id(&intent.id)
    {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    if u64::from(bot.speed) < pos.0.hex_distance(intent.position) {
        return OperationResult::InvalidInput;
    }

    match terrain.get_by_id(&intent.position) {
        Some(model::TileTerrainType::Wall) => OperationResult::InvalidInput,
        _ => OperationResult::Ok,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Bot, PositionComponent};
    use crate::storage::Storage;
    use crate::tables::Table;
    use caolo_api::point::Point;

    #[test]
    fn test_move_intent_fails_if_node_is_occupied() {
        let mut storage = Storage::new();
        storage.add_entity_table::<Bot>(Table::default_inmemory());
        storage.add_entity_table::<PositionComponent>(Table::default_inmemory());

        let id = storage.insert_entity();

        storage.entity_table_mut::<Bot>().insert(
            id,
            Bot {
                speed: 2,
                owner_id: None,
            },
        );

        storage
            .entity_table_mut::<PositionComponent>()
            .insert(id, PositionComponent(Point::new(12, 13)));

        let intent = MoveIntent {
            bot: 69,
            position: Point::new(42, 42),
        };

        intent
            .execute(&mut storage)
            .expect_err("Expected move to fail");
    }
}
