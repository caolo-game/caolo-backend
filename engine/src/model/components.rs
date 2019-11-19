use super::*;

pub use caolo_api::terrain::TileTerrainType;

#[derive(Debug, Clone)]
pub struct Bot {
    pub owner_id: Option<UserId>,
    pub speed: u8,
}

#[derive(Debug, Clone)]
pub struct Structure {
    pub owner_id: Option<UserId>,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct PositionComponent(pub Point);

#[derive(Debug, Clone, Copy)]
pub struct EnergyComponent {
    pub energy: u16,
    pub energy_max: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct SpawnComponent {
    pub time_to_spawn: u8,
    pub spawning: Option<EntityId>,
}

#[derive(Debug, Clone, Copy)]
pub struct HpComponent {
    pub hp: u16,
    pub hp_max: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyRegenComponent {
    pub amount: u16,
}

/// Represent time to decay of bots
/// On decay the bot will loose hp
#[derive(Debug, Clone, Copy)]
pub struct DecayComponent {
    pub hp_amount: u16,
    pub eta: u8,
    pub t: u8,
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

// TODO: more resource types
#[derive(Debug, Clone, Default)]
pub struct CarryComponent {
    pub carry: u16,
    pub carry_max: u16,
}

#[derive(Debug, Clone)]
pub struct EntityScript {
    pub script_id: ScriptId,
}
