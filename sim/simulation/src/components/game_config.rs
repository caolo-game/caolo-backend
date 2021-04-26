use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub world_radius: u32,
    pub room_radius: u32,
    pub execution_limit: u32,
    pub target_tick_ms: u64,
    /// Unique ID of this world instance
    pub queen_tag: String,
    /// maximum number of steps pathfinding can test
    pub path_finding_limit: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            execution_limit: 128,
            target_tick_ms: 100,
            queen_tag: uuid::Uuid::new_v4().to_string(),
            world_radius: 32,
            room_radius: 50,
            path_finding_limit: 1000,
        }
    }
}
