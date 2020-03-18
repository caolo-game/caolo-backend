use crate::intents::{check_spawn_intent, SpawnIntent as InnerSpawnIntent};
use crate::model::OperationResult;
use crate::systems::script_execution::ScriptExecutionData;
use cao_lang::prelude::*;

/// Given a SpawnIntent as input instructs the current spawn to spawn a new Bot
pub fn spawn(vm: &mut VM<ScriptExecutionData>, intent: TPointer) -> Result<(), ExecutionError> {
    unimplemented!()
    // let intent = match vm.get_value::<SpawnIntent>(intent) {
    //     None => {
    //         log::error!("spawn intent not set");
    //         return Err(ExecutionError::MissingArgument);
    //     }
    //     Some(i) => i,
    // };
    //
    // let storage = vm.get_aux().storage();
    // let userid = vm.get_aux().userid();
    //
    // let check = check_spawn_intent(&intent, userid, storage);
    // match check {
    //     OperationResult::Ok => {}
    //     _ => {
    //         return vm.set_value(check);
    //     }
    // }
    //
    // let intent = InnerSpawnIntent {
    //     id: intent.id,
    //     owner_id: vm.get_aux().userid(),
    //     bot: intent.bot,
    // };
    //
    // vm.get_aux_mut().intents_mut().spawn_intents.push(intent);
    //
    // vm.set_value(OperationResult::Ok)
}
