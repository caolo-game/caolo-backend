mod dropoff_intent_system;
mod log_intent_system;
mod mine_intent_system;
mod move_intent_system;
mod spawn_intent_system;

use self::dropoff_intent_system::DropoffSystem;
use self::log_intent_system::LogSystem;
use self::mine_intent_system::MineSystem;
use self::move_intent_system::MoveSystem;
use self::spawn_intent_system::SpawnSystem;
use crate::intents::{Intents, MoveIntent};
use crate::profile;
use crate::storage::{
    views::{HasNew, HasNewMut},
    Storage,
};
use rayon::prelude::*;

pub trait IntentExecutionSystem<'a> {
    type Mut: HasNewMut;
    type Const: HasNew<'a>;
    type Intent;

    fn execute(&mut self, m: Self::Mut, c: Self::Const, intents: &[Self::Intent]);
}

/// Executes all intents in order of priority (as defined by this system)
pub fn execute_intents(mut intents: Intents, storage: &mut Storage) {
    profile!("execute_intents");

    pre_process_move_intents(&mut intents.move_intents);

    let mut move_sys = MoveSystem;
    move_sys.execute(
        From::from(storage as &mut _),
        From::from(storage as &_),
        intents.move_intents.as_slice(),
    );

    let mut mine_sys = MineSystem;
    mine_sys.execute(
        From::from(storage as &mut _),
        From::from(storage as &_),
        intents.mine_intents.as_slice(),
    );

    let mut dropoff_sys = DropoffSystem;
    dropoff_sys.execute(
        From::from(storage as &mut _),
        From::from(storage as &_),
        intents.dropoff_intents.as_slice(),
    );

    let mut spawn_sys = SpawnSystem;
    spawn_sys.execute(
        From::from(storage as &mut _),
        From::from(storage as &_),
        intents.spawn_intents.as_slice(),
    );

    let mut log_sys = LogSystem;
    log_sys.execute(
        From::from(storage as &mut _),
        From::from(storage as &_),
        intents.log_intents.as_slice(),
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
