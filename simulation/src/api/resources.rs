use crate::model::{self, EntityId, Resource as ResourceComponent};
use crate::storage::Storage;
use caolo_api::resources::{Mineral, Resource};

pub const MAX_SEARCH_RADIUS: u32 = 60;

pub fn build_resource(
    id: EntityId,
    resource: ResourceComponent,
    storage: &Storage,
) -> Option<caolo_api::resources::Resource> {
    let positions = storage.entity_table::<model::PositionComponent>();
    let energy = storage.entity_table::<model::EnergyComponent>();
    match resource {
        ResourceComponent::Mineral => {
            let pos = positions.get_by_id(&id).or_else(|| {
                error!("Mineral {:?} has no position", id);
                None
            })?;
            let energy = energy.get_by_id(&id).or_else(|| {
                error!("Mineral {:?} has no energy", id);
                None
            })?;

            let mineral = Mineral::new(id, pos.0, energy.energy, energy.energy_max);

            Some(Resource::Mineral(mineral))
        }
    }
}
