use caolo_api::{EntityId as EId, ScriptId as SId, UserId as UId};

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash)]
pub struct EntityTime(pub EId, pub u64);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash)]
pub struct EntityId(pub EId);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy)]
pub struct ScriptId(pub SId);

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Hash)]
pub struct UserId(pub UId);
