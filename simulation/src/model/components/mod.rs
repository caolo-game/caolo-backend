mod resources;
pub use resources::*;

use super::terrain::TileTerrainType;
use super::{geometry::Point, EntityId, ScriptId, UserId};
use crate::tables::{BTreeTable, Component, MortonTable, SpatialKey2d, TableId, VecTable};
use serde_derive::Serialize;

/// For tables that store entity ids as values
#[derive(Debug, Clone, Serialize, Copy)]
pub struct EntityComponent(pub EntityId);
impl<Id: SpatialKey2d + Send + Sync> Component<Id> for EntityComponent {
    type Table = MortonTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Bot {}

impl Component<EntityId> for Bot {
    type Table = VecTable<EntityId, Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct Structure {}

impl<Id: TableId> Component<Id> for Structure {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnedEntity {
    pub owner_id: UserId,
}

impl<Id: TableId> Component<Id> for OwnedEntity {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Serialize)]
pub struct PositionComponent(pub Point);
impl Component<EntityId> for PositionComponent {
    type Table = VecTable<EntityId, Self>;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EnergyComponent {
    pub energy: u16,
    pub energy_max: u16,
}
impl<Id: TableId> Component<Id> for EnergyComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SpawnComponent {
    pub time_to_spawn: i16,
    pub spawning: Option<EntityId>,
}
impl<Id: TableId> Component<Id> for SpawnComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HpComponent {
    pub hp: u16,
    pub hp_max: u16,
}
impl<Id: TableId> Component<Id> for HpComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EnergyRegenComponent {
    pub amount: u16,
}
impl<Id: TableId> Component<Id> for EnergyRegenComponent {
    type Table = BTreeTable<Id, Self>;
}

/// Represent time to decay of bots
/// On decay the bot will loose hp
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DecayComponent {
    pub hp_amount: u16,
    pub eta: u8,
    pub t: u8,
}
impl<Id: TableId> Component<Id> for DecayComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct SpawnBotComponent {
    pub bot: Bot,
}
impl<Id: TableId> Component<Id> for SpawnBotComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CarryComponent {
    pub carry: u16,
    pub carry_max: u16,
}
impl<Id: TableId> Component<Id> for CarryComponent {
    type Table = BTreeTable<Id, Self>;
}

/// Entity - Script join table
#[derive(Debug, Clone, Serialize)]
pub struct EntityScript {
    pub script_id: ScriptId,
}
impl<Id: TableId> Component<Id> for EntityScript {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub payload: Vec<String>,
}
impl<Id: TableId> Component<Id> for LogEntry {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct TerrainComponent(pub TileTerrainType);
impl<Id: SpatialKey2d + Send + Sync> Component<Id> for TerrainComponent {
    type Table = MortonTable<Id, Self>;
}

/// Entities with Scripts
#[derive(Debug, Clone, Serialize)]
pub struct ScriptComponent(pub cao_lang::CompiledProgram);
impl<Id: TableId> Component<Id> for ScriptComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize)]
pub struct UserComponent;
impl<Id: TableId> Component<Id> for UserComponent {
    type Table = BTreeTable<Id, Self>;
}
