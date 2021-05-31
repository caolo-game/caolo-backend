use crate::components::SayComponent;
use crate::indices::*;
use crate::intents::{Intents, SayIntent};
use crate::profile;
use crate::storage::views::{UnsafeView, UnwrapViewMut};
use std::mem::take;
use tracing::trace;

type Mut = (
    UnsafeView<EntityId, SayComponent>,
    UnwrapViewMut<EmptyKey, Intents<SayIntent>>,
);

pub fn say_intents_update((mut say_table, mut intents): Mut, (): ()) {
    profile!("SayIntentSystem update");

    let intents = take(&mut intents.0);

    for intent in intents {
        trace!("inserting say entry {:?}", intent);
        say_table.insert_or_update(intent.entity, SayComponent(intent.payload));
    }
}
