use super::*;
use crate::EntityId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mineral {
    pub id: EntityId,
    pub position: Point,
    pub energy: u16,
    pub energy_max: u16,
}

impl Mineral {
    pub fn new(id: EntityId, position: Point, energy: u16, energy_max: u16) -> Self {
        Self {
            id,
            position,
            energy,
            energy_max,
        }
    }
}

impl cao_lang::traits::AutoByteEncodeProperties for Mineral {}
