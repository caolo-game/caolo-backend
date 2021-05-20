use crate::{components::SayPayload, indices::EntityId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogIntent {
    pub entity: EntityId,
    pub payload: String,
    pub time: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SayIntent {
    pub entity: EntityId,
    pub payload: SayPayload,
}
