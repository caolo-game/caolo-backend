use super::*;
use crate::tables::{BTreeTable, Component, QuadtreeTable, SpatialKey2d, TableId};

pub use caolo_api::terrain::TileTerrainType;

#[derive(Debug, Clone)]
pub struct Bot {}

impl<Id: TableId> Component<Id> for Bot {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct Structure {}

impl<Id: TableId> Component<Id> for Structure {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct OwnedEntity {
    pub owner_id: UserId,
}

impl<Id: TableId> Component<Id> for OwnedEntity {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub struct PositionComponent(pub Point);
impl<Id: TableId> Component<Id> for PositionComponent {
    type Table = BTreeTable<Id, Self>;
}

impl std::ops::Add for PositionComponent {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(Point {
            x: self.0.x + other.0.x,
            y: self.0.y + other.0.y,
        })
    }
}

impl SpatialKey2d for PositionComponent {
    fn get_axis(&self, axis: u8) -> i32 {
        match axis {
            0 => self.0.x,
            1 => self.0.y,
            _ => unreachable!(),
        }
    }

    fn new(x: i32, y: i32) -> Self {
        Self(Point { x, y })
    }

    fn dist(&self, other: &Self) -> u32 {
        use std::convert::TryFrom;
        u32::try_from(self.0.hex_distance(other.0)).expect("Distance to fit in 32 bits")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyComponent {
    pub energy: u16,
    pub energy_max: u16,
}
impl<Id: TableId> Component<Id> for EnergyComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnComponent {
    pub time_to_spawn: u8,
    pub spawning: Option<EntityId>,
}
impl<Id: TableId> Component<Id> for SpawnComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct HpComponent {
    pub hp: u16,
    pub hp_max: u16,
}
impl<Id: TableId> Component<Id> for HpComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyRegenComponent {
    pub amount: u16,
}
impl<Id: TableId> Component<Id> for EnergyRegenComponent {
    type Table = BTreeTable<Id, Self>;
}

/// Represent time to decay of bots
/// On decay the bot will loose hp
#[derive(Debug, Clone, Copy)]
pub struct DecayComponent {
    pub hp_amount: u16,
    pub eta: u8,
    pub t: u8,
}
impl<Id: TableId> Component<Id> for DecayComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Resource {
    Mineral,
}

#[derive(Debug, Clone)]
pub struct SpawnBotComponent {
    pub bot: Bot,
}
impl<Id: TableId> Component<Id> for SpawnBotComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Default)]
pub struct CarryComponent {
    pub carry: u16,
    pub carry_max: u16,
}
impl<Id: TableId> Component<Id> for CarryComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct EntityScript {
    pub script_id: ScriptId,
}
impl<Id: TableId> Component<Id> for EntityScript {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub payload: Vec<String>,
}
impl<Id: TableId> Component<Id> for LogEntry {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TerrainComponent(pub TileTerrainType);
impl<Id: SpatialKey2d + Send + Sync> Component<Id> for TerrainComponent {
    type Table = QuadtreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct ResourceComponent(pub Resource);
impl<Id: TableId> Component<Id> for ResourceComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct ScriptComponent(pub caolo_api::Script);
impl<Id: TableId> Component<Id> for ScriptComponent {
    type Table = BTreeTable<Id, Self>;
}

#[derive(Debug, Clone)]
pub struct UserComponent(pub UserData);
impl<Id: TableId> Component<Id> for UserComponent {
    type Table = BTreeTable<Id, Self>;
}
