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
