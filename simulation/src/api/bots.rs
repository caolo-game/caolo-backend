use super::*;
use crate::{
    intents::{check_move_intent, MoveIntent},
    model::{bots, components, geometry::point::Point, OperationResult},
    profile,
    systems::pathfinding,
};

/// In: x, y coordinates
/// Out: OperationResult
pub fn move_bot(
    vm: &mut VM<ScriptExecutionData>,
    point: TPointer,
) -> Result<Object, ExecutionError> {
    profile!("move_bot");

    let entity = vm.get_aux().entityid();
    debug!("moving bot {:?}", entity);

    let point: Point = vm.get_value(point).ok_or_else(|| {
        error!("move_bot called without a point");
        ExecutionError::InvalidArgument
    })?;
    let storage = vm.get_aux().storage();

    let positions = storage.point_table::<components::EntityComponent>();
    let terrain = storage.point_table::<components::TerrainComponent>();

    let botpos = storage
        .entity_table::<components::PositionComponent>()
        .get_by_id(&entity)
        .ok_or_else(|| {
            error!("entity {:?} does not have position component!", entity);
            ExecutionError::InvalidArgument
        })?;

    let mut path = Vec::with_capacity(1000);
    if let Err(e) = pathfinding::find_path(botpos.0, point, positions, terrain, 1000, &mut path) {
        debug!("pathfinding failed {:?}", e);
        return vm.set_value(OperationResult::InvalidTarget);
    }

    let intent = match path.get(0) {
        Some(position) => {
            bots::MoveIntent {
                id: entity,
                position: *position, // TODO: cache path
            }
        }
        None => {
            debug!("Entity {:?} is trying to move to its own position", entity);
            return vm.set_value(OperationResult::InvalidTarget);
        }
    };
    let userid = vm.get_aux().userid().expect("userid to be set");

    let checkresult = check_move_intent(&intent, userid, From::from(storage as &_));
    let result = vm.set_value(checkresult);

    vm.get_aux_mut()
        .intents_mut()
        .move_intents
        .push(MoveIntent {
            bot: intent.id,
            position: intent.position,
        });

    result
}
