use crate::intents::{Intent, SpawnIntent as InnerSpawnIntent};
use crate::model::{EntityId, SpawnComponent};
use crate::systems::execution::ScriptExecutionData;
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

    match storage
        .entity_table::<SpawnComponent>()
        .get_by_id(&EntityId(intent.id))
        .map(|c| c.spawning.is_none())
    {
        None => {
            return Err(ExecutionError::TaskFailure(format!(
                "Spawn called on an entity {} that is not a spawn!",
                intent.id
            )));
        }
        Some(false) => {
            log::warn!("spawn called on entity {} that is busy!", intent.id);
            let s = vm.set_value_at(output, OperationResult::InvalidInput);
            return Ok(s);
        }
        _ => {}
    }

    let intent = InnerSpawnIntent {
        id: EntityId(intent.id),
        owner_id: vm.get_aux().userid(),
        bot: intent.bot,
    };

    vm.get_aux_mut().intents_mut().push(Intent::Spawn(intent));

    let s = vm.set_value_at(output, OperationResult::Ok);
    return Ok(s);
}
