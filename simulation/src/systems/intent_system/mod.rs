mod dropoff_intent_system;
mod log_intent_system;
mod mine_intent_system;
mod move_intent_system;
mod path_cache_intent_system;
mod spawn_intent_system;

use self::dropoff_intent_system::DropoffSystem;
use self::log_intent_system::LogSystem;
use self::mine_intent_system::MineSystem;
use self::move_intent_system::MoveSystem;
use self::path_cache_intent_system::{PopPathCacheSystem, UpdatePathCacheSystem};
use self::spawn_intent_system::SpawnSystem;
use crate::intents::{Intents, MoveIntent};
use crate::profile;
use crate::storage::views::{FromWorld, FromWorldMut};
use crate::World;
use rayon::prelude::*;

pub trait IntentExecutionSystem<'a> {
    type Mut: FromWorldMut;
    type Const: FromWorld<'a>;
    type Intent;

    fn execute(&mut self, m: Self::Mut, c: Self::Const, intents: &[Self::Intent]);
}

/// Executes all intents in order of priority (as defined by this system)
pub fn execute_intents(mut intents: Intents, storage: &mut World) {
    profile!("execute_intents");

    pre_process_move_intents(&mut intents.move_intents);

    let intents = &intents;

    rayon::scope(move |s| {
        let move_sys = executor(MoveSystem, storage);
        let mine_sys = executor(MineSystem, storage);
        let dropoff_sys = executor(DropoffSystem, storage);
        s.spawn(move |_| {
            move_sys(intents);
            mine_sys(intents);
            dropoff_sys(intents);
        });

        let spawn_sys = executor(SpawnSystem, storage);
        s.spawn(move |_| {
            spawn_sys(intents);
        });

        let log_sys = executor(LogSystem, storage);
        s.spawn(move |_| {
            log_sys(intents);
        });

        let update_cache_sys = executor(UpdatePathCacheSystem, storage);
        let pop_path_cache_sys = executor(PopPathCacheSystem, storage);
        s.spawn(move |_| {
            update_cache_sys(intents);
            pop_path_cache_sys(intents);
        });
    });
}

fn executor<'a, 'b, T, Sys>(
    mut sys: Sys,
    storage: *mut World,
) -> impl FnOnce(&'b Intents) -> () + 'a
where
    'b: 'a,
    T: 'a,
    &'a Intents: Into<&'a [T]>,
    Sys: IntentExecutionSystem<'a, Intent = T> + 'a,
{
    let storage = unsafe { &mut *storage };
    let mutable = Sys::Mut::new(storage);
    let immutable = Sys::Const::new(storage);

    move |intents| {
        sys.execute(mutable, immutable, intents.into());
    }
}

/// Remove duplicate positions.
/// Replaces duplicate positions with log intents
fn pre_process_move_intents(move_intents: &mut Vec<MoveIntent>) {
    if move_intents.len() < 2 {
        // 0 and 1 long vectors do not have duplicates
        return;
    }
    move_intents.par_sort_unstable_by_key(|intent| intent.position);
    let len = move_intents.len();
    let mut last_pos = len - 1;
    // move in reverse order as we want to remove invalid intents as we move
    for index in (0..len - 2).rev() {
        let a = &move_intents[last_pos];
        let b = &move_intents[index];
        if a.position == b.position {
            debug!("Duplicated position in move intent, removing {:?}", a);
            move_intents.remove(last_pos);
        }
        last_pos = index;
    }
}
