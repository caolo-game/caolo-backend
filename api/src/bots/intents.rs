use super::*;
use crate::resources::ResourceType;
use crate::rmps::{self, Serializer};
use crate::EntityId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveIntent {
    pub id: EntityId,
    pub position: Point,
}

impl MoveIntent {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode MoveIntent {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MineIntent {
    pub id: EntityId,
    pub target: EntityId,
}

impl MineIntent {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode MineIntent {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropoffIntent {
    pub id: EntityId,
    pub target: EntityId,
    pub amount: u16,
    pub ty: ResourceType,
}

impl DropoffIntent {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode MineIntent {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}
