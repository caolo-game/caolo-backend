use crate::components::LogEntry;
use crate::indices::*;
use crate::intents::{Intents, LogIntent};
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapViewMut};
use crate::tables::Table;
use std::mem::take;
use tracing::trace;

type Mut = (
    UnsafeView<EntityTime, LogEntry>,
    UnwrapViewMut<EmptyKey, Intents<LogIntent>>,
);

pub fn update((mut log_table, mut intents): Mut, (): ()) {
    profile!("LogIntentSystem update");

    let intents = take(&mut intents.0);

    for intent in intents {
        trace!("inserting log entry {:?}", intent);
        let id = EntityTime(intent.entity, intent.time);
        // use delete to move out of the data structure, then we'll move it back in
        // this should be cheaper than cloning all the time, because of the inner vectors
        match log_table.delete(id) {
            Some(mut entry) => {
                entry.payload.extend_from_slice(intent.payload.as_slice());
                log_table.insert_or_update(id, entry);
            }
            None => {
                let entry = LogEntry {
                    payload: intent.payload,
                };
                log_table.insert_or_update(id, entry);
            }
        };
    }
}
