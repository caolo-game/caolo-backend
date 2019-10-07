use super::*;
use crate::bots::Bot;
use crate::rmps::{self, Serializer};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: Bot,
}

impl SpawnIntent {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode SpawnIntent {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}
