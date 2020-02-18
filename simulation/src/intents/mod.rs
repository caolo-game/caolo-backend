//! Actions, world updates the clients _intented_ to be executed.
//!
mod dropoff_intent;
mod log_intent;
mod mine_intent;
mod move_intent;
mod spawn_intent;

pub use self::dropoff_intent::*;
pub use self::log_intent::*;
pub use self::mine_intent::*;
pub use self::move_intent::*;
pub use self::spawn_intent::*;

use crate::model::EntityId;

#[derive(Debug, Clone, Default)]
pub struct Intents {
    pub move_intents: Vec<MoveIntent>,
    pub spawn_intents: Vec<SpawnIntent>,
    pub mine_intents: Vec<MineIntent>,
    pub dropoff_intents: Vec<DropoffIntent>,
    pub log_intents: Vec<LogIntent>,
}

impl Intents {
    pub fn merge(&mut self, other: Intents) -> &mut Self {
        self.move_intents.extend_from_slice(&other.move_intents);
        self.spawn_intents.extend_from_slice(&other.spawn_intents);
        self.mine_intents.extend_from_slice(&other.mine_intents);
        self.dropoff_intents
            .extend_from_slice(&other.dropoff_intents);
        self.log_intents.extend_from_slice(&other.log_intents);
        self
    }

    pub fn new() -> Self {
        Self::default()
    }
}
