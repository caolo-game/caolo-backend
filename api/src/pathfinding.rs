use crate::point::Point;
use crate::rmps::{self, Serializer};

/// Holds a path find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub path: Vec<Point>,
}

impl Path {
    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Bot {:?}", e);
            "Deserialize failed"
        })
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(512);
        <Self as serde::Serialize>::serialize(&self, &mut Serializer::new(&mut buffer)).unwrap();
        buffer
    }
}
