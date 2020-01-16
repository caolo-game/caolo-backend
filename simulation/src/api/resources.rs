use super::*;
use crate::model::{
    self, EntityComponent, EntityId, PositionComponent, Resource, ResourceComponent,
};
use crate::profile;
use crate::storage::Storage;
use caolo_api::resources::Mineral;

pub const MAX_SEARCH_RADIUS: u32 = 256;

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
    vm: &mut VM<ScriptExecutionData>,
    _: (),
) -> Result<Object, ExecutionError> {
    profile!("find_closest_resource_by_range");

    let entityid = vm.get_aux().entityid();
    let storage = vm.get_aux().storage();

    let position = match storage
        .entity_table::<PositionComponent>()
        .get_by_id(&entityid)
    {
        Some(p) => p,
        None => {
            debug!("{:?} has no PositionComponent", entityid);
            return vm.set_value((OperationResult::InvalidInput,));
        }
    };

    let mut candidates = Vec::with_capacity(MAX_SEARCH_RADIUS as usize * 2);
    storage.point_table::<EntityComponent>().find_by_range(
        &position.0,
        MAX_SEARCH_RADIUS,
        &mut candidates,
    );

    let resources = storage.entity_table::<ResourceComponent>();

    candidates.retain(|(_pos, entityid)| resources.get_by_id(&entityid.0).is_some());
    match candidates
        .iter()
        .min_by_key(|(pos, _)| pos.hex_distance(position.0))
    {
        None => vm.set_value((OperationResult::OperationFailed,)),
        Some((pos, _entity)) => {
            // move out of the result to free the storage borrow
            let pos = *pos;
            vm.set_value((OperationResult::Ok, pos))
        }
    }
}
