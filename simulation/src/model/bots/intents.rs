use crate::model::components::Resource;
use crate::model::EntityId;

#[derive(Clone, Debug)]
pub struct DropoffIntent {
    pub id: EntityId,
    pub target: EntityId,
    pub amount: u16,
    pub ty: Resource,
}
