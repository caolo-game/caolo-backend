use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ClientMessage {
    /// Authenthicate using a brearer token
    AuthToken(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum InputPayload {
    UpdateScript(UpdateScript),
    UpdateEntityScript(UpdateEntityScript),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct InputMsg {
    pub msg_id: Uuid,
    pub payload: InputPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct UpdateEntityScript {
    pub user_id: Uuid,
    pub entity_id: u32,
    pub script_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct UpdateScript {
    pub user_id: Uuid,
    pub script_id: Uuid,
    pub compiled_script: CompiledScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CompiledScript {
    pub bytecode: Vec<u8>,
    pub labels: HashMap<i32, Label>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Label {
    pub block: u32,
    pub myself: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Function {
    pub name: String,
    pub description: String,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub params: Vec<String>,
}

impl Function {
    pub fn from_str_parts(
        name: &str,
        description: &str,
        input: &[&str],
        output: &[&str],
        params: &[&str],
    ) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            input: input.iter().map(|x| x.to_string()).collect(),
            output: output.iter().map(|x| x.to_string()).collect(),
            params: params.iter().map(|x| x.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Schema {
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct AxialPoint {
    pub q: i32,
    pub r: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct WorldPosition {
    pub room: AxialPoint,
    pub room_pos: AxialPoint,
    pub absolute_pos: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Bot {
    pub id: u32,
    pub position: WorldPosition,
    pub owner: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Structure {
    pub id: u32,
    pub position: WorldPosition,
    pub owner: Option<Uuid>,
    pub payload: StructurePayload,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum StructurePayload {
    Spawn(StructureSpawn),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct StructureSpawn {
    pub time_to_spawn: i32,
    pub spawning: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Resource {
    pub id: u32,
    pub ty: ResourceType,
    pub position: WorldPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ResourceType {
    Energy { energy: u32, energy_max: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct WorldState {
    pub bots: Vec<Bot>,
    pub logs: Vec<LogEntry>,
    pub resources: Vec<Resource>,
    pub structures: Vec<Structure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct LogEntry {
    pub entity_id: u32,
    pub time: u64,
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct RoomTerrainMessage {
    pub tiles: Vec<Tile>,
    pub room_properties: RoomProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct RoomProperties {
    pub room_radius: u32,
    pub room_id: AxialPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct Tile {
    pub position: WorldPosition,
    pub ty: TerrainType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum TerrainType {
    Empty,
    Wall,
    Plain,
    Bridge,
}
