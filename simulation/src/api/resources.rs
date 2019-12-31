use super::*;
use crate::model::{self, EntityId, Resource, ResourceComponent};
use crate::profile;
use crate::storage::Storage;
use caolo_api::resources::Mineral;

pub const MAX_SEARCH_RADIUS: u32 = 60;

pub fn build_resource(
    id: EntityId,
    resource: ResourceComponent,
    storage: &Storage,
) -> Option<caolo_api::resources::Resource> {
    let positions = storage.entity_table::<model::PositionComponent>();
    let energy = storage.entity_table::<model::EnergyComponent>();
    match resource.0 {
        Resource::Mineral => {
            let pos = positions.get_by_id(&id).or_else(|| {
                error!("Mineral {:?} has no position", id);
                None
            })?;
            let energy = energy.get_by_id(&id).or_else(|| {
                error!("Mineral {:?} has no energy", id);
                None
            })?;

            let mineral = Mineral::new(id.0, pos.0, energy.energy, energy.energy_max);
            let mineral = caolo_api::resources::Resource::Mineral(mineral);
            Some(mineral)
        }
    }
}

pub fn find_closest_resource_by_range(
    _vm: &mut VM<ScriptExecutionData>,
    _: (),
    _output: TPointer,
) -> Result<usize, ExecutionError> {
    profile!("find_closest_resource_by_range");
    // let entityid = vm.get_aux().entityid();
    // let storage = vm.get_aux().storage();
    //
    // let positions = storage.entity_table::<PositionComponent>();
    // let resources = storage.entity_table::<ResourceComponent>();
    unimplemented!()
}
