pub mod game_config;

mod bot_components;
mod resources;
mod rooms;
pub use bot_components::*;
pub use resources::*;
pub use rooms::*;

use crate::{
    indices::{EntityId, Room, UserId, WorldPosition},
    prelude::Axial,
    tables::{
        btree::BTreeTable, dense::DenseVecTable, flag::SparseFlagTable, morton::MortonTable,
        Component, MortonMortonTable, TableId,
    },
};
use cao_lang::prelude::CaoProgram;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Currently does nothing as Cao-Lang not yet supports history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptHistoryEntry {
    pub entity_id: EntityId,
    pub time: u64,
}

/// Currently does nothing as Cao-Lang not yet supports history
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScriptHistory(());
impl Component<EntityId> for ScriptHistory {
    type Table = DenseVecTable<EntityId, Self>;
}

/// For tables that store entity ids as values
#[derive(Debug, Clone, Serialize, Deserialize, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntityComponent(pub EntityId);
impl Component<Axial> for EntityComponent {
    type Table = MortonTable<Self>;
}
impl Component<WorldPosition> for EntityComponent {
    type Table = MortonMortonTable<Self>;
}

/// Has a body so it's not `null` when serializing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Structure;
impl<Id: TableId> Component<Id> for Structure {
    type Table = SparseFlagTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OwnedEntity {
    pub owner_id: UserId,
}

impl Component<EntityId> for OwnedEntity {
    type Table = DenseVecTable<EntityId, Self>;
}

impl Component<Axial> for OwnedEntity {
    type Table = MortonTable<Self>;
}

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionComponent(pub WorldPosition);
impl Component<EntityId> for PositionComponent {
    type Table = DenseVecTable<EntityId, Self>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnergyComponent {
    pub energy: u16,
    pub energy_max: u16,
}
impl<Id: TableId> Component<Id> for EnergyComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnComponent {
    /// Time to spawn the current entity
    pub time_to_spawn: i16,
    /// Currently spawning entity
    pub spawning: Option<EntityId>,
}

impl<Id: TableId> Component<Id> for SpawnComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnQueueComponent {
    /// Entities waiting for spawn
    pub queue: VecDeque<EntityId>,
}

impl<Id: TableId> Component<Id> for SpawnQueueComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HpComponent {
    pub hp: u16,
    pub hp_max: u16,
}
impl Component<EntityId> for HpComponent {
    type Table = DenseVecTable<EntityId, Self>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnergyRegenComponent {
    pub amount: u16,
}
impl<Id: TableId> Component<Id> for EnergyRegenComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpawnBotComponent {
    pub bot: Bot,
}

impl<Id: TableId> Component<Id> for SpawnBotComponent {
    type Table = BTreeTable<Id, Self>;
}

// TODO:
// maximize number of logs stored
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub payload: Vec<String>,
}
impl<Id: TableId> Component<Id> for LogEntry {
    type Table = BTreeTable<Id, Self>;
}

/// Entities with Scripts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptComponent(pub CaoProgram);
impl<Id: TableId> Component<Id> for ScriptComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserComponent;
impl<Id: TableId> Component<Id> for UserComponent {
    type Table = SparseFlagTable<Id, Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProperties {
    pub level: u16,
}

impl Default for UserProperties {
    fn default() -> Self {
        Self { level: 1 }
    }
}

impl Component<UserId> for UserProperties {
    type Table = BTreeTable<UserId, Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rooms(pub Vec<Room>);
impl<Id: TableId> Component<Id> for Rooms {
    type Table = BTreeTable<Id, Self>;
}
