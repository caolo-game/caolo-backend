use super::*;
use crate::{
    intents::{check_move_intent, MoveIntent},
    model::{self, EntityId, Point},
    profile,
    storage::Storage,
    systems::pathfinding,
};
use caolo_api::OperationResult;

/// In: x, y coordinates
/// Out: OperationResult
pub fn move_bot(
    vm: &mut VM<ScriptExecutionData>,
    point: TPointer,
    output: TPointer,
) -> Result<usize, ExecutionError> {
    profile!("mode_bot");

    let point: Point = vm.get_value(point).ok_or_else(|| {
        error!("move_bot called without a point");
        ExecutionError::InvalidArgument
    })?;
    let entity = vm.get_aux().entityid();
    let storage = vm.get_aux().storage();

    let positions = storage.point_table::<model::EntityComponent>();
    let terrain = storage.point_table::<model::TerrainComponent>();

    let botpos = storage
        .entity_table::<model::PositionComponent>()
        .get_by_id(&entity)
        .ok_or_else(|| {
            error!("entity {:?} does not have position component!", entity);
            ExecutionError::InvalidArgument
        })?;

    let path = pathfinding::find_path(botpos.0, point, positions, terrain, 5000).map_err(|e| {
        // TODO this isnt really an error
        error!("pathfinding failed {:?}", e);
        ExecutionError::TaskFailure(format!(
            "Could not find path from {:?} to {:?}",
            botpos, point
        ))
    })?;

    let intent = if let Some(position) = path.get(0).cloned() {
        caolo_api::bots::MoveIntent {
            id: entity.0,
            position, // TODO: cache path
        }
    } else {
        debug!("Entity {:?} is trying to move to its own position", entity);
        return Ok(0);
    };
    let userid = vm.get_aux().userid().expect("userid to be set");

    let result = {
        let checkresult = check_move_intent(&intent, userid, storage);
        match checkresult {
            OperationResult::Ok => 0,
            _ => vm.set_value_at(output, checkresult),
        }
    };

    vm.get_aux_mut()
        .intents_mut()
        .move_intents
        .push(MoveIntent {
            bot: EntityId(intent.id),
            position: intent.position,
        });

    return Ok(result);
}

pub fn build_bot(id: EntityId, storage: &Storage) -> Option<caolo_api::bots::Bot> {
    profile!("build_bot");

    let bot = storage.entity_table::<model::Bot>().get_by_id(&id);
    if bot.is_none() {
        debug!(
            "Bot {:?} could not be built because it has no bot component",
            id
        );
        return None;
    }

    let pos = storage
        .entity_table::<model::PositionComponent>()
        .get_by_id(&id)
        .or_else(|| {
            debug!("Bot {:?} could not be built because it has no position", id);
            None
        })?;

    let carry = storage
        .entity_table::<model::CarryComponent>()
        .get_by_id(&id)
        .unwrap_or_else(|| &model::CarryComponent {
            carry: 0,
            carry_max: 0,
        });

    let owner_id = storage.entity_table::<model::OwnedEntity>().get_by_id(&id);

    Some(caolo_api::bots::Bot {
        id: id.0,
        owner_id: owner_id.map(|id| id.owner_id.0),
        position: pos.0,
        carry: carry.carry,
        carry_max: carry.carry_max,
    })
}
