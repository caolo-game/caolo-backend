mod spawn_intent;
pub use spawn_intent::*;

use crate::rmps::{self, Serializer};
use crate::EntityId;
use crate::{point::Point, OperationResult, UserId};

mod external {
    extern "C" {
        pub fn _get_my_structures_len() -> i32;
        pub fn _get_my_structures(ptr: *mut u8) -> i32;
        pub fn _send_spawn_intent(ptr: *const u8, len: i32) -> i32;
    }
}

pub fn get_my_structures() -> Vec<Structure> {
    let len = unsafe { external::_get_my_structures_len() as usize };
    if len == 0 {
        return vec![];
    }
    let mut data = vec![0; len * std::mem::size_of::<Structure>() * 2];
    let len = unsafe { external::_get_my_structures(data.as_mut_ptr()) as usize };
    let structures =
        Structures::deserialize(&data[0..len]).expect("Failed to deserialize structures");
    structures.structures
}

pub fn send_spawn_intent(intent: SpawnIntent) -> OperationResult {
    let data = intent.serialize();
    let len = data.len();

    let result = unsafe { external::_send_spawn_intent(data.as_ptr(), len as i32) };

    OperationResult::from(result)
}

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
