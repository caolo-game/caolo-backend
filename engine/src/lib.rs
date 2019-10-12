#[macro_use]
extern crate log;

pub mod api;
pub mod model;
pub mod storage;
pub mod tables;

mod intents;
mod systems;
mod utils;

use caolo_api::{EntityId, UserId};
use systems::execution::{execute_intents, execute_scripts};
use systems::execute_world_update;

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

    unimplemented!();

    info!("Compiling scripts done");
}
