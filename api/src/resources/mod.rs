mod minerals;

pub use minerals::*;

use crate::external;
use crate::point::Circle;
use crate::point::Point;
use crate::rmps::{self, Serializer};
use crate::OperationResult;

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

    pub fn deserialize(buffer: &[u8]) -> Result<Self, &'static str> {
        rmps::from_slice(buffer).map_err(|e| {
            println!("Failed to decode Resources {:?}", e);
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

pub fn find_resources_in_range(range: Circle) -> Result<Resources, OperationResult> {
    let mut buff = vec![0; range.radius as usize * 6 * std::mem::size_of::<Mineral>()];

    let result = unsafe {
        external::_find_resources_in_range(
            range.center.x,
            range.center.y,
            range.radius as i32,
            buff.as_mut_ptr(),
        )
    };

    if result < 0 {
        let result = OperationResult::from(result);
        return Err(result);
    }

    let len = result;
    if len == 0 {
        return Ok(Resources::default());
    }

    let resources =
        Resources::deserialize(&buff[..len as usize]).expect("Failed to deserialize resources");

    Ok(resources)
}
