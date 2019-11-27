use super::*;
use crate::EntityId;

/// Represents a Bot in the game world
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bot {
    pub id: EntityId,
    pub position: Point,
    pub owner_id: Option<UserId>,

    pub carry: u16,
    pub carry_max: u16,
}

impl cao_lang::traits::AutoByteEncodeProperties for Bot {}
