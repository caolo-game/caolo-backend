use crate::indices::EntityId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogIntent {
    pub entity: EntityId,
    pub payload: String,
    pub time: u64,
}
