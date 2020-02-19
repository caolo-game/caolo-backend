use super::*;
use crate::{
    intents::{check_mine_intent, check_move_intent, MineIntent, MoveIntent},
    model::{
        components, components::ResourceComponent, geometry::point::Point, EntityId,
        OperationResult, UserId,
    },
    profile,
    storage::Storage,
    systems::pathfinding,
};

pub fn mine_resource(
    vm: &mut VM<ScriptExecutionData>,
    entity_id: TPointer,
) -> Result<Object, ExecutionError> {
    profile!("mine_resource");
    let entity_id: EntityId = vm.get_value(entity_id).ok_or_else(|| {
        error!("mine_resource called without an entity_id");
        ExecutionError::InvalidArgument
    })?;
    let aux = vm.get_aux();
    let storage = aux.storage();
    if storage
        .entity_table::<ResourceComponent>()
        .get_by_id(&entity_id)
        .is_none()
    {
        warn!("mine_resource called on an entity that is not a resource");
        return Err(ExecutionError::InvalidArgument);
    }

    let intent = MineIntent {
        bot: aux.entityid(),
        resource: entity_id,
    };

    let userid = aux.userid().expect("userid to be set");

    let checkresult = check_mine_intent(&intent, userid, storage.into());
    if let OperationResult::Ok = checkresult {
        vm.get_aux_mut().intents_mut().mine_intents.push(intent);
    }

    vm.set_value(checkresult)
}

pub fn approach_entity(
    vm: &mut VM<ScriptExecutionData>,
    target: TPointer,
) -> Result<Object, ExecutionError> {
    profile!("approach_entity");

    let entity = vm.get_aux().entityid();

    let target: EntityId = vm.get_value(target).ok_or_else(|| {
        error!("approach_entity called without an entity");
        ExecutionError::InvalidArgument
    })?;

    let storage = vm.get_aux().storage();

    let targetpos = match storage
        .entity_table::<components::PositionComponent>()
        .get_by_id(&target)
    {
        Some(x) => x,
        None => {
            warn!("entity {:?} does not have position component!", target);
            return vm.set_value(OperationResult::InvalidInput);
        }
    };
    let userid = vm.get_aux().userid().expect("userid to be set");

    let checkresult = match move_to_pos(entity, targetpos.0, userid, storage) {
        Ok(intent) => {
            vm.get_aux_mut().intents_mut().move_intents.push(intent);
            OperationResult::Ok
        }
        Err(e) => e,
    };
    vm.set_value(checkresult)
}

pub fn move_bot_to_position(
    vm: &mut VM<ScriptExecutionData>,
    point: TPointer,
) -> Result<Object, ExecutionError> {
    profile!("move_bot_to_position");

    let entity = vm.get_aux().entityid();

    let point: Point = vm.get_value(point).ok_or_else(|| {
        error!("move_bot called without a point");
        ExecutionError::InvalidArgument
    })?;
    let storage = vm.get_aux().storage();
    let userid = vm.get_aux().userid().expect("userid to be set");

    let checkresult = match move_to_pos(entity, point, userid, storage) {
        Ok(intent) => {
            vm.get_aux_mut().intents_mut().move_intents.push(intent);
            OperationResult::Ok
        }
        Err(e) => e,
    };
    vm.set_value(checkresult)
}

fn move_to_pos(
    bot: EntityId,
    to: Point,
    userid: UserId,
    storage: &Storage,
) -> Result<MoveIntent, OperationResult> {
    let botpos = storage
        .entity_table::<components::PositionComponent>()
        .get_by_id(&bot)
        .ok_or_else(|| {
            warn!("entity {:?} does not have position component!", bot);
            OperationResult::InvalidInput
        })?;
    let mut path = Vec::with_capacity(1000);
    if let Err(e) = pathfinding::find_path(botpos.0, to, storage.into(), 1000, &mut path) {
        debug!("pathfinding failed {:?}", e);
        return Err(OperationResult::InvalidTarget);
    }

    // TODO: cache path
    let intent = match path.pop() {
        Some(position) => MoveIntent {
            bot,
            position: position,
        },
        None => {
            debug!("Entity {:?} is trying to move to its own position", bot);
            return Err(OperationResult::InvalidTarget);
        }
    };
    let checkresult = check_move_intent(&intent, userid, storage.into());
    if let OperationResult::Ok = checkresult {
        Ok(intent)
    } else {
        Err(checkresult)
    }
}
