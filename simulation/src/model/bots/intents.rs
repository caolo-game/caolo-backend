use crate::model::components::Resource;
use crate::model::geometry::point::Point;
use crate::model::EntityId;

#[derive(Clone, Debug, Default)]
pub struct MoveIntent {
    pub id: EntityId,
    pub position: Point,
}

#[derive(Clone, Debug, Default)]
pub struct MineIntent {
    pub id: EntityId,
    pub target: EntityId,
}

#[derive(Clone, Debug)]
pub struct DropoffIntent {
    pub id: EntityId,
    pub target: EntityId,
    pub amount: u16,
    pub ty: Resource,
}
