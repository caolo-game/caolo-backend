use crate::indices::{RoomPosition, ScriptId, WorldPosition};
use arrayvec::ArrayVec;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub struct MeleeAttackComponent {
    pub strength: u16,
}

/// Has a body so it's not `null` when serializing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Bot;

/// Represent time to decay of bots
/// On decay the bot will loose hp
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DecayComponent {
    pub hp_amount: u16,
    pub interval: u8,
    pub time_remaining: u8,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CarryComponent {
    pub carry: u16,
    pub carry_max: u16,
}

/// Entity - Script join table
#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
#[serde(rename_all = "camelCase")]
pub struct EntityScript(pub ScriptId);

unsafe impl Send for EntityScript {}

pub const PATH_CACHE_LEN: usize = 64;
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PathCacheComponent {
    pub target: WorldPosition,
    pub path: ArrayVec<RoomPosition, PATH_CACHE_LEN>,
}
