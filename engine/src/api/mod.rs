//! Methods that are exported to the WASM clients
//!
//! Methods that may fail return an OperationResult or the length of the result in bytes.
//!
mod bots;
mod pathfinding;
mod resources;
mod structures;
pub use self::bots::*;
pub use self::pathfinding::*;
pub use self::resources::*;
pub use self::stdout::*;
pub use self::structures::*;
use crate::{get_current_user_id, get_intents_mut, get_storage, intents};
use caolo_api::{self, OperationResult};
use rand::Rng;
use wasmer_runtime::{func, imports, Ctx, ImportObject};

pub fn make_import() -> ImportObject {
    imports! {
        "env" => {
            "_print" => func!(_print),
            "_rand_range" => func!(_rand_range),
            "_get_my_bots" => func!(_get_my_bots),
            "_get_my_bots_len" => func!(_get_my_bots_len),
            "_send_move_intent" => func!(_send_move_intent),
            "_send_mine_intent" => func!(_send_mine_intent),
            "_send_dropoff_intent" => func!(_send_dropoff_intent),
            "_get_my_structures" => func!(_get_my_structures),
            "_get_my_structures_len" => func!(_get_my_structures_len),
            "_send_spawn_intent" => func!(_send_spawn_intent),
            "_get_max_search_radius" => func!(_get_max_search_radius),
            "_find_path" => func!(_find_path),
            "_get_max_path_length" => func!(_get_max_path_length),
            "_find_resources_in_range" => func!(_find_resources_in_range),
        },
    }
}

#[no_mangle]
pub fn _rand_range(_ctx: &mut Ctx, from: i32, to: i32) -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(from, to)
}

mod stdout {
    use super::*;

    #[no_mangle]
    pub fn _print(ctx: &mut Ctx, ptr: i32, len: i32) {
        let userid = unsafe { get_current_user_id(ctx) };

        let memory = ctx.memory(0);

        let ptr = ptr as usize;
        let len = len as usize;
        let str_vec: Vec<_> = memory.view()[ptr..ptr + len]
            .iter()
            .map(|cell| cell.get())
            .collect();
        let string = std::str::from_utf8(&str_vec).expect("Failed to parse utf8 string");
        debug!("[{}]: {}", userid, string);
    }
}

fn save_bytes_to_memory(ctx: &mut Ctx, ptr: usize, len: usize, data: &[u8]) {
    let memory = ctx.memory(0);
    for (byte, cell) in data.iter().zip(memory.view()[ptr..ptr + len].iter()) {
        cell.set(*byte);
    }
}

fn read_bytes(ctx: &Ctx, ptr: usize, len: usize) -> Vec<u8> {
    let memory = ctx.memory(0);
    memory.view()[ptr..ptr + len]
        .iter()
        .map(|cell| cell.get())
        .collect()
}
