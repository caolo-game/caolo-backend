use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorldState {
    pub rooms: HashMap<AxialPoint, RoomState>,
    pub game_config: Value,
    pub room_properties: Value,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum StructureType {
    Spawn,
}
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct RoomState {
    pub bots: Vec<Value>,
    pub resources: Vec<Value>,
    pub structures: Vec<Value>,
}
