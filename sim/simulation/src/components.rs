pub mod game_config;

mod bot_components;
mod resources;
mod rooms;
mod script_components;
pub use bot_components::*;
pub use resources::*;
pub use rooms::*;
pub use script_components::*;

use crate::indices::{EntityId, Room, UserId, WorldPosition};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// For tables that store entity ids as values
#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntityComponent(pub EntityId);

/// Has a body so it's not `null` when serializing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Structure;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OwnedEntity {
    pub owner_id: UserId,
}

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionComponent(pub WorldPosition);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnergyComponent {
    pub energy: u16,
    pub energy_max: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnComponent {
    /// Time to spawn the current entity
    pub time_to_spawn: i16,
    /// Currently spawning entity
    pub spawning: Option<EntityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnQueueComponent {
    /// Entities waiting for spawn
    pub queue: VecDeque<EntityId>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HpComponent {
    pub hp: u16,
    pub hp_max: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnergyRegenComponent {
    pub amount: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnBotComponent {
    pub bot: Bot,
}

// TODO:
// maximize number of logs stored
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub payload: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserComponent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProperties {
    pub level: u16,
}

impl Default for UserProperties {
    fn default() -> Self {
        Self { level: 1 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rooms(pub Vec<Room>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespawnTimer(pub i32);
