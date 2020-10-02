use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorldState {
    pub rooms: HashMap<AxialPoint, RoomState>,
    pub logs: Vec<LogEntry>,
    /// key = entity id
    pub script_history: HashMap<u32, ScriptHistoryEntry>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AxialPoint {
    pub q: i32,
    pub r: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct WorldPosition {
    pub room: AxialPoint,
    pub room_pos: AxialPoint,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Bot {
    pub id: u32,
    pub position: WorldPosition,
    pub owner: Option<Uuid>,

    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Structure {
    pub id: u32,
    pub position: WorldPosition,
    pub owner: Option<Uuid>,
    pub payload: StructurePayload,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub enum StructurePayload {
    Spawn(StructureSpawn),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct StructureSpawn {
    pub time_to_spawn: u32,
    pub spawning: u32,
    pub energy: u32,
    pub energy_max: u32,
    pub energy_regen: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum StructureType {
    Spawn,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Resource {
    pub id: u32,
    pub ty: ResourceType,
    pub position: WorldPosition,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub enum ResourceType {
    Energy { energy: u32, energy_max: u32 },
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RoomState {
    pub bots: Vec<Bot>,
    pub resources: Vec<Resource>,
    pub structures: Vec<Structure>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LogEntry {
    pub entity_id: u32,
    pub time: u64,
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ScriptHistoryEntry {
    pub entity_id: u32,
    pub payload: Vec<i64>,
}
