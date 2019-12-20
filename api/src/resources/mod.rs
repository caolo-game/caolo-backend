mod minerals;

pub use minerals::*;

use crate::point::Point;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResourceType {
    Mineral,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resources {
    pub resources: Vec<Resource>,
}

impl Resources {
    pub fn new(resources: Vec<Resource>) -> Self {
        Self { resources }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", content = "data", rename_all = "camelCase")]
pub enum Resource {
    Mineral(Mineral),
}

impl Resource {
    pub fn position(&self) -> Point {
        match self {
            Resource::Mineral(m) => m.position,
        }
    }
}
