#[macro_use]
extern crate log;

pub mod api;
pub mod model;
pub mod storage;
pub mod tables;

mod intents;
mod systems;
mod utils;

use api::make_import;
use caolo_api::{EntityId, UserId};
use systems::execution::{execute_intents, execute_scripts};
use systems::execute_world_update;
use wasmer_runtime::compile;
use wasmer_runtime_core::vm::InternalField;

static INTENTS: InternalField = InternalField::allocate();
static STORAGE: InternalField = InternalField::allocate();
static CURRENT_USER_ID: InternalField = InternalField::allocate();

unsafe fn get_intents_mut(ctx: &mut wasmer_runtime::Ctx) -> &mut Vec<intents::Intent> {
    let ptr = ctx.get_internal(&INTENTS) as *mut _;
    &mut *ptr
}

// Mutable storage is not allowed in a wasm context!
unsafe fn get_storage(ctx: &wasmer_runtime::Ctx) -> &storage::Storage {
    let ptr = ctx.get_internal(&STORAGE) as *const storage::Storage;
    &(*ptr)
}

unsafe fn get_current_user_id(ctx: &wasmer_runtime::Ctx) -> &UserId {
    let ptr = ctx.get_internal(&CURRENT_USER_ID) as *const _;
    &*ptr
}

pub fn forward(storage: &mut storage::Storage) -> Result<(), Box<dyn std::error::Error>> {
    let users = storage.users().collect::<Vec<_>>();

    compile_scripts(&users, storage);
    let final_intents = execute_scripts(&users, storage);

    storage.signal_done(&final_intents);

    info!("Executing intents {}", final_intents.len());
    execute_intents(final_intents, storage);
    info!("Executing intents - done");
    info!("Executing systems update");
    execute_world_update(storage);
    info!("Executing systems update - done");

    Ok(())
}

fn compile_scripts(userids: &[UserId], storage: &mut storage::Storage) {
    use rayon::prelude::*;

    info!("Compiling scripts");

    let compiled = userids
        .par_iter()
        .filter_map(|userid| {
            debug!("Compiling script [{}]", userid);

            let udata = storage.user_table::<model::UserData>().get_by_id(userid)?;
            if udata.compiled.is_some() {
                debug!("Script of user [{}] is already compiled", userid);
                None?;
            }
            let wasm = udata.script.as_ref()?;

            let compiled = compile(wasm)
                .map_err(|e| {
                    error!("Failed to compile script of user [{}] {:?}", userid, e);
                })
                .ok()?;

            Some((userid, compiled))
        })
        .collect::<Vec<_>>();

    for (userid, compiled) in compiled {
        let compiled = compiled.cache().unwrap().serialize().unwrap();
        let mut row = storage
            .user_table::<model::UserData>()
            .get_by_id(userid)
            .unwrap_or_default();
        row.compiled = Some(compiled);
        storage
            .user_table_mut::<model::UserData>()
            .insert(*userid, row);
    }
    info!("Compiling scripts done");
}
