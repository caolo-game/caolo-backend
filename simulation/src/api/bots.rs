use super::*;
use crate::{
    intents::{
        check_dropoff_intent, check_mine_intent, check_move_intent, DropoffIntent, MineIntent,
        MoveIntent,
    },
    model::{
        components::{self, Resource, ResourceComponent},
        geometry::point::Point,
        EntityId, OperationResult, UserId,
    },
    profile,
    storage::views::FromWorld,
    systems::pathfinding,
    World,
};
use std::convert::TryFrom;

pub fn unload(
    vm: &mut VM<ScriptExecutionData>,
    (amount, ty, structure): (i32, Resource, TPointer),
) -> Result<Object, ExecutionError> {
    profile!("unload");

    let amount = TryFrom::try_from(amount).map_err(|e| {
        debug!("unload called with invalid amount: {}", e);
        ExecutionError::InvalidArgument
    })?;
    let structure: EntityId = vm.get_value(structure).ok_or_else(|| {
        warn!("upload called without a structure");
        ExecutionError::InvalidArgument
    })?;

    let aux = vm.get_aux();
    let storage = aux.storage();
    let entityid = aux.entityid();
    let userid = aux.userid().expect("userid to be set");

    let dropoff_intent = DropoffIntent {
        bot: entityid,
        amount,
        ty,
        structure,
    };

    let checkresult = check_dropoff_intent(&dropoff_intent, userid, FromWorld::new(storage));
    if let OperationResult::Ok = checkresult {
        vm.get_aux_mut()
            .intents_mut()
            .dropoff_intents
            .push(dropoff_intent);
    }
    vm.set_value(checkresult)
}

pub fn mine_resource(
    vm: &mut VM<ScriptExecutionData>,
    entityid: TPointer,
) -> Result<Object, ExecutionError> {
    profile!("mine_resource");

    let entityid: EntityId = vm.get_value(entityid).ok_or_else(|| {
        warn!("approach_entity called without a target");
        ExecutionError::InvalidArgument
    })?;

    let aux = vm.get_aux();
    let storage = aux.storage();
    let userid = aux.userid().expect("userid to be set");

    if storage
        .view::<EntityId, ResourceComponent>()
        .get_by_id(&entityid)
        .is_none()
    {
        warn!("mine_resource called on an entity that is not a resource");
        return Err(ExecutionError::InvalidArgument);
    }

    let intent = MineIntent {
        bot: aux.entityid(),
        resource: entityid,
    };

    let checkresult = check_mine_intent(&intent, userid, FromWorld::new(storage));
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

    let target: EntityId = vm.get_value(target).ok_or_else(|| {
        warn!("approach_entity called without a target");
        ExecutionError::InvalidArgument
    })?;

    let aux = vm.get_aux();
    let entity = aux.entityid();
    let storage = aux.storage();
    let userid = aux.userid().expect("userid to be set");

    let targetpos = match storage
        .view::<EntityId, components::PositionComponent>()
        .reborrow()
        .get_by_id(&target)
    {
        Some(x) => x,
        None => {
            warn!("entity {:?} does not have position component!", target);
            return vm.set_value(OperationResult::InvalidInput);
        }
    };

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

    let aux = vm.get_aux();
    let entity = aux.entityid();
    let storage = aux.storage();
    let userid = aux.userid().expect("userid to be set");

    let point: Point = vm.get_value(point).ok_or_else(|| {
        warn!("move_bot called without a point");
        ExecutionError::InvalidArgument
    })?;

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
    storage: &World,
) -> Result<MoveIntent, OperationResult> {
    let botpos = storage
        .view::<EntityId, components::PositionComponent>()
        .reborrow()
        .get_by_id(&bot)
        .ok_or_else(|| {
            warn!("entity {:?} does not have position component!", bot);
            OperationResult::InvalidInput
        })?;
    let mut path = Vec::with_capacity(1000);
    if let Err(e) = pathfinding::find_path(botpos.0, to, FromWorld::new(storage), 1000, &mut path) {
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
    let checkresult = check_move_intent(&intent, userid, FromWorld::new(storage));
    if let OperationResult::Ok = checkresult {
        Ok(intent)
    } else {
        Err(checkresult)
    }
}
