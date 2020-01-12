use crate::intents::Intent;
use crate::profile;
use crate::storage::Storage;
use rayon::prelude::*;

pub fn execute_intents(mut intents: Vec<Intent>, storage: &mut Storage) {
    profile!("execute_intents");

    intents.par_sort_unstable_by_key(|i| i.priority());

    pre_process_move_intents(&mut intents, storage.time());

    for intent in intents {
        debug!("Executing intent {:?}", intent);
        intent
            .execute(storage)
            .map_err(|e| {
                debug!("Intent execution failed {:?}", e);
            })
            .unwrap_or_default();
    }
}

/// Remove duplicate positions.
/// Replaces duplicate positions with log intents
fn pre_process_move_intents(intents: &mut Vec<Intent>, current_time: u64) {
    let mut move_intents_to = 0;
    // filter out move intents
    let move_intents = intents
        .iter()
        .enumerate()
        .filter(|(i, int)| match int {
            Intent::Move(_) => {
                move_intents_to = *i;
                true
            }
            _ => false,
        })
        .count();
    let move_intents_from = move_intents_to + 1 - move_intents;
    let move_intents = &mut intents[move_intents_from..=move_intents_to];
    // sort move intents by their positions
    move_intents.sort_unstable_by_key(|int| match int {
        Intent::Move(int) => int.position,
        _ => unreachable!("MoveIntents must have a unique priority!"),
    });
    let len = move_intents.len();
    let mut last_pos = 0;
    for i in 1..len {
        // traverse in the opposite direction
        match (&move_intents[last_pos], &move_intents[i]) {
            (Intent::Move(a), Intent::Move(b)) => {
                if a.position == b.position {
                    move_intents[last_pos] =
                        Intent::new_log(a.bot, "Move failed: occupied".to_owned(), current_time);
                }
            }
            _ => unreachable!(),
        }
        last_pos = i;
    }
}
