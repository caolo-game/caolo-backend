use crate::{
    intents,  model, profile, storage::Storage, UserId, 
};
use rayon::prelude::*;

pub type ExecutionResult = Result<Vec<intents::Intent>, String>;

/// Must be called from a tokio runtime!
/// Returns the intents that are expected to be executed
pub fn execute_scripts(userids: &[UserId], storage: &Storage) -> Vec<intents::Intent> {
    profile!("execute_scripts");

    userids
        .par_iter()
        .map(|userid| {
            execute_single_script(&userid, storage)
                .map_err(|e| {
                    error!("Execution failure userid: [{}] {:?}", &userid, e);
                })
                .unwrap_or_else(|_| Vec::with_capacity(0))
        })
        .reduce(
            || Vec::with_capacity(userids.len() * 16),
            |mut out, ints| {
                out.extend(ints.into_iter());
                out
            },
        )
}

pub fn execute_single_script(userid: &UserId, storage: &Storage) -> ExecutionResult {
    profile!("execute_single_script");
    debug!("Starting script_execution of user [{}]", userid);

    let udata = storage.user_table::<model::UserData>().get_by_id(userid);
    if udata.is_none() {
        debug!("User {:?} has no user data!", userid);
        return Err("User not found".into());
    }
    let udata = udata.unwrap();
    let wasm = &udata.compiled;
    if wasm.is_none() {
        debug!("User {:?} has no compiledscript!", userid);
        return Err("User has no compiled script".into());
    }


    unimplemented!();
    let intents = vec![];

    debug!("User [{}] script was executed successfully", userid);

    Ok(intents)
}
