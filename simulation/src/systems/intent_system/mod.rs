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

    let mut move_sys = MoveSystem;
    execute(&mut move_sys, storage, intents);

    let mut mine_sys = MineSystem;
    execute(&mut mine_sys, storage, intents);

    let mut dropoff_sys = DropoffSystem;
    execute(&mut dropoff_sys, storage, intents);

    let mut spawn_sys = SpawnSystem;
    execute(&mut spawn_sys, storage, intents);

    let mut log_sys = LogSystem;
    execute(&mut log_sys, storage, intents);

    // first update the cache, then pop
    let mut path_cache_sys = UpdatePathCacheSystem;
    execute(&mut path_cache_sys, storage, intents);

    let mut path_cache_sys = PopPathCacheSystem;
    execute(&mut path_cache_sys, storage, intents);
}

#[inline]
fn execute<'a, T, Sys>(sys: &mut Sys, storage: &'a mut World, intents: &'a Intents)
where
    T: 'a,
    &'a Intents: Into<&'a [T]>,
    Sys: IntentExecutionSystem<'a, Intent = T>,
{
    sys.execute(
        FromWorldMut::new(storage),
        FromWorld::new(storage as &_),
        intents.into(),
    );
}

/// Remove duplicate positions.
/// Replaces duplicate positions with log intents
fn pre_process_move_intents(move_intents: &mut Vec<MoveIntent>) {
    if move_intents.len() < 2 {
        // 0 and 1 long vectors do not have duplicates
        return;
    }
    // sort move intents by their positions
    move_intents.par_sort_unstable_by_key(|int| int.position);
    let len = move_intents.len();
    let mut last_pos = len - 1;
    // move in reverse order as we want to remove invalid intents as we move
    for i in (0..len - 2).rev() {
        let a = &move_intents[last_pos];
        let b = &move_intents[i];
        if a.position == b.position {
            debug!("Duplicated position in move intent, removing {:?}", a);
            move_intents.remove(last_pos);
        }
        last_pos = i;
    }
}
