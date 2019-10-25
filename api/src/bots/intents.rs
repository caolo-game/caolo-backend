use super::*;
use crate::resources::ResourceType;
use crate::EntityId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveIntent {
    pub id: EntityId,
    pub position: Point,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MineIntent {
    pub id: EntityId,
    pub target: EntityId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropoffIntent {
    pub id: EntityId,
    pub target: EntityId,
    pub amount: u16,
    pub ty: ResourceType,
}
