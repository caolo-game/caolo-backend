mod spawn_intent;
pub use spawn_intent::*;

use crate::rmps::{self, Serializer};
use crate::EntityId;
use crate::{point::Point, OperationResult, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structures {
    pub structures: Vec<Structure>,
}

impl Structures {
    pub fn new(structures: Vec<Structure>) -> Self {
        Self { structures }
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Structures {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", content = "data", rename_all = "camelCase")]
pub enum Structure {
    Spawn(Spawn),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spawn {
    pub id: EntityId,
    pub position: Point,
    pub owner_id: Option<UserId>,

    pub energy: u16,
    pub energy_max: u16,

    pub time_to_spawn: u8,
    pub spawning: Option<EntityId>,

    pub hp: u16,
    pub hp_max: u16,
}
