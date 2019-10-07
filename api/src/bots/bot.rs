use super::*;
use crate::rmps::{self, Serializer};
use crate::EntityId;

/// Represents a Bot in the game world
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
    pub id: EntityId,
    pub position: Point,
    pub speed: u8,
    pub owner_id: Option<UserId>,

    pub carry: u16,
    pub carry_max: u16,
}

impl Bot {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Bot {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }

    pub fn new(
        id: EntityId,
        position: Point,
        speed: u8,
        owner_id: Option<UserId>,
        carry: u16,
        carry_max: u16,
    ) -> Self {
        Self {
            id,
            position,
            speed,
            owner_id,
            carry,
            carry_max,
        }
    }
}
