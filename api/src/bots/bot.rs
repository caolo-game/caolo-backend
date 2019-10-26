use super::*;
use crate::EntityId;

/// Represents a Bot in the game world
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
    pub id: EntityId,
    pub position: Point,
    pub speed: u8,
    pub owner_id: Option<UserId>,

    pub carry: u16,
    pub carry_max: u16,
}

impl Bot {
    pub fn new(
        id: EntityId,
        position: Point,
        speed: u8,
        owner_id: Option<UserId>,
        carry: u16,
        carry_max: u16,
    ) -> Self {
        Self {
            id,
            position,
            speed,
            owner_id,
            carry,
            carry_max,
        }
    }
}

impl cao_lang::traits::AutoByteEncodeProperties for Bot {}
