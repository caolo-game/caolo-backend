use crate::intents::{check_spawn_intent, SpawnIntent as InnerSpawnIntent};
use crate::model::EntityId;
use crate::systems::script_execution::ScriptExecutionData;
use cao_lang::prelude::*;
use caolo_api::structures::SpawnIntent;
use caolo_api::OperationResult;

/// Given a SpawnIntent as input instructs the current spawn to spawn a new Bot
pub fn spawn(
    vm: &mut VM<ScriptExecutionData>,
    intent: TPointer,
    output: TPointer,
) -> Result<usize, ExecutionError> {
    let intent = match vm.get_value::<SpawnIntent>(intent) {
        None => {
            log::error!("spawn intent not set");
            return Err(ExecutionError::MissingArgument);
        }
        Some(i) => i,
    };

    let storage = vm.get_aux().storage();
    let userid = vm.get_aux().userid();

    let check = check_spawn_intent(&intent, userid, storage);
    match check {
        OperationResult::Ok => {}
        _ => {
            let s = vm.set_value_at(output, check);
            return Ok(s);
        }
    }

    let intent = InnerSpawnIntent {
        id: EntityId(intent.id),
        owner_id: vm.get_aux().userid(),
        bot: intent.bot,
    };

    vm.get_aux_mut().intents_mut().spawn_intents.push(intent);

    let s = vm.set_value_at(output, OperationResult::Ok);
    return Ok(s);
}
