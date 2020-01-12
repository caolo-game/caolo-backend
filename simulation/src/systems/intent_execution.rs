use crate::intents::{Intents, MoveIntent};
use crate::profile;
use crate::storage::Storage;
use rayon::prelude::*;

pub fn execute_intents(mut intents: Intents, storage: &mut Storage) {
    profile!("execute_intents");

    macro_rules! execute {
        ($intent: ident) => {
            debug!("Executing intent {:?}", $intent);
            $intent
                .execute(storage)
                .map_err(|e| {
                    debug!("Intent execution failed {:?}", e);
                })
                .unwrap_or_default();
        };
    };

    pre_process_move_intents(&mut intents.move_intents);
    for intent in intents.move_intents {
        execute!(intent);
    }
    for intent in intents.mine_intents {
        execute!(intent);
    }
    for intent in intents.dropoff_intents {
        execute!(intent);
    }
    for intent in intents.spawn_intents {
        execute!(intent);
    }
    for intent in intents.log_intents {
        execute!(intent);
    }
}

/// Remove duplicate positions.
/// Replaces duplicate positions with log intents
fn pre_process_move_intents(move_intents: &mut Vec<MoveIntent>) {
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
