use super::*;
use crate::intents::{self, check_move_intent};
use crate::model::{self, EntityId, Point};
use crate::profile;
use crate::storage::Storage;
use caolo_api::OperationResult;

/// In: x, y coordinates
/// Out: OperationResult
pub fn move_bot(
    vm: &mut VM<ScriptExecutionData>,
    point: TPointer,
    output: TPointer,
) -> Result<usize, ExecutionError> {
    profile!("_send_move_intent");

    let point: Point = vm.get_value(point).ok_or_else(|| {
        error!("move_bot called without a point");
        ExecutionError::InvalidArgument
    })?;
    let intent = caolo_api::bots::MoveIntent {
        id: vm.get_aux().entityid(),
        position: point,
    };
    let userid = Default::default();
    let storage = vm.get_aux().storage();

    let result = {
        let checkresult = check_move_intent(&intent, userid, storage);
        match checkresult {
            OperationResult::Ok => 0,
            _ => vm.set_value_at(output, checkresult),
        }
    };

    vm.get_aux_mut()
        .intents_mut()
        .push(intents::Intent::new_move(intent.id, intent.position));

    return Ok(result);
}

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