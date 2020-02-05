use crate::tables::SerialId;
use caolo_api::{EntityId as EId, ScriptId as SId, UserId as UId};
use serde_derive::Serialize;

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityTime(pub EId, pub u64);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct EntityId(pub EId);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize)]
pub struct ScriptId(pub SId);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash, Serialize)]
pub struct UserId(pub UId);

impl SerialId for EntityId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
