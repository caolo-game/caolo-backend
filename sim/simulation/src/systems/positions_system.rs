use crate::components::{EntityComponent, PositionComponent};
use crate::indices::{EntityId, WorldPosition};
use crate::profile;
use crate::storage::views::{UnsafeView, View};
use tracing::{debug, error};

type Mut = UnsafeView<WorldPosition, EntityComponent>;
type Const<'a> = (View<'a, EntityId, PositionComponent>,);

/// Reset the entity positions table
pub fn positions_update(mut position_entities: Mut, (entity_positions,): Const) {
    profile!("PositionSystem update");
    debug!("update positions system called");

    let mut positions = entity_positions
        .iter()
        .map(|(id, PositionComponent(pos))| (*pos, EntityComponent(id)))
        .collect::<Vec<_>>();

    position_entities.clear();
    position_entities
        .extend_from_slice(positions.as_mut_slice())
        .map_err(|e| {
            error!("Failed to rebuild position_entities table {:?}", e);
        })
        .ok();
}
