use crate::{
    intents, make_import, model, profile, storage::Storage, UserId, CURRENT_USER_ID, INTENTS,
    STORAGE,
};
use rayon::prelude::*;
use wasmer_runtime::{cache::Artifact, Func};
use wasmer_runtime_core::load_cache_with;

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

    let import_object = make_import();

    let module = udata
        .compiled
        .and_then(|compiled| {
            let artifact = Artifact::deserialize(&compiled).ok()?;
            unsafe { load_cache_with(artifact, &wasmer_runtime::default_compiler()).ok() }
        })
        .ok_or_else(|| "Failed to get a user script")?;

    let mut instance = module
        .instantiate(&import_object)
        .map_err(|e| format!("{:?}", e))?;

    let mut intents = Vec::<intents::Intent>::with_capacity(16);

    let ctx = instance.context_mut();
    ctx.set_internal(&INTENTS, &mut intents as *mut _ as u64);
    ctx.set_internal(&STORAGE, storage as *const Storage as u64);
    ctx.set_internal(&CURRENT_USER_ID, userid as *const _ as u64);

    let mainfn: Func<(), ()> = instance.func("run").map_err(|e| format!("{:?}", e))?;

    mainfn
        .call()
        .map_err(|e| format!("Run call failure {:?}", e))?;

    debug!("User [{}] script was executed successfully", userid);

    Ok(intents)
}
