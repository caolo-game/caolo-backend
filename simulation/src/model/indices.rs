use crate::tables::SerialId;
use serde_derive::Serialize;

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityTime(pub u32, pub u64);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize)]
pub struct ScriptId(pub uuid::Uuid);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct UserId(pub uuid::Uuid);

impl SerialId for EntityId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
