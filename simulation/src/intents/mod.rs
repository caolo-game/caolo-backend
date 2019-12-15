//! Actions _intented_ to be executed by clients
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
use crate::storage::Storage;
use caolo_api::bots::Bot;
use caolo_api::point::Point;

pub type IntentResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum Intent {
    Move(MoveIntent),
    Spawn(SpawnIntent),
    Mine(MineIntent),
    Dropoff(DropoffIntent),
    Log(LogIntent),
}

impl Intent {
    /// Higher priority should be executed first
    pub fn priority(&self) -> u16 {
        match self {
            Intent::Move(_) => 1,
            Intent::Spawn(_) => 10,
            Intent::Mine(_) => 9,
            Intent::Dropoff(_) => 9,
            Intent::Log(_) => 0,
        }
    }

    pub fn execute(self, storage: &mut Storage) -> IntentResult {
        match self {
            Intent::Move(intent) => intent.execute(storage),
            Intent::Spawn(intent) => intent.execute(storage),
            Intent::Mine(intent) => intent.execute(storage),
            Intent::Dropoff(intent) => intent.execute(storage),
            Intent::Log(intent) => intent.execute(storage),
        }
    }

    pub fn new_move(bot: EntityId, position: Point) -> Self {
        Intent::Move(MoveIntent { bot, position })
    }

    pub fn new_spawn(bot: Bot, structure_id: EntityId) -> Self {
        Intent::Spawn(SpawnIntent {
            id: structure_id,
            bot,
        })
    }

    pub fn new_mine(bot: EntityId, resource: EntityId) -> Self {
        Intent::Mine(MineIntent { bot, resource })
    }

    pub fn new_log(entity: EntityId, payload: String, time: u64) -> Self {
        Intent::Log(LogIntent {
            entity,
            payload,
            time,
        })
    }

    pub fn new_dropoff(
        bot: EntityId,
        structure: EntityId,
        amount: u16,
        ty: crate::model::ResourceType,
    ) -> Self {
        Intent::Dropoff(DropoffIntent {
            bot,
            structure,
            amount,
            ty,
        })
    }
}
