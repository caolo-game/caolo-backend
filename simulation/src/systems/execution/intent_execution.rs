use crate::intents::Intent;
use crate::profile;
use crate::storage::Storage;
use rayon::prelude::*;

pub fn execute_intents(mut intents: Vec<Intent>, storage: &mut Storage) {
    profile!("execute_intents");

    intents.par_sort_by_key(|i| i.priority());

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
