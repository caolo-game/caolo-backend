use super::*;
use crate::rmps::{self, Serializer};
use crate::EntityId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Mineral {
    pub id: EntityId,
    pub position: Point,
    pub energy: u16,
    pub energy_max: u16,
}

impl Mineral {
    pub fn new(id: EntityId, position: Point, energy: u16, energy_max: u16) -> Self {
        Self {
            id,
            position,
            energy,
            energy_max,
        }
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Mineral {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}
