use crate::model::{
    self,
    components::{self, EntityComponent, PositionComponent},
    geometry::Point,
    terrain, EntityId, OperationResult,
};
use crate::storage::views::View;

#[derive(Debug, Clone)]
pub struct MoveIntent {
    pub bot: EntityId,
    pub position: Point,
}

pub fn check_move_intent(
    intent: &MoveIntent,
    userid: model::UserId,
    (owner_ids, positions, bots, terrain, entity_positions): (
        View<EntityId, components::OwnedEntity>,
        View<EntityId, PositionComponent>,
        View<EntityId, components::Bot>,
        View<Point, components::TerrainComponent>,
        View<Point, EntityComponent>,
    ),
) -> OperationResult {
    let id = intent.bot;
    match bots.get_by_id(&id) {
        Some(_) => {
            let owner_id = owner_ids.get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => return OperationResult::InvalidInput,
    };

    let pos = match positions.get_by_id(&id) {
        Some(pos) => pos,
        None => {
            debug!("Bot has no position");
            return OperationResult::InvalidInput;
        }
    };

    // TODO: bot speed component?
    if 1 < pos.0.hex_distance(intent.position) {
        debug!(
            "Bot move target {:?} is out of range of bot position {:?} and velocity {:?}",
            intent.position, pos, 1
        );
        return OperationResult::InvalidInput;
    }

    match terrain.get_by_id(&intent.position) {
        Some(components::TerrainComponent(terrain::TileTerrainType::Wall)) => {
            debug!("Position is occupied by terrain");
            return OperationResult::InvalidInput;
        }
        _ => {}
    }
    if let Some(entity) = entity_positions.get_by_id(&intent.position) {
        debug!("Position is occupied by another entity {:?}", entity);
        return OperationResult::InvalidInput;
    }
    OperationResult::Ok
}
