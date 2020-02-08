use super::*;
use crate::model::{
    components::{EntityComponent, PositionComponent, ResourceComponent},
};
use crate::profile;

pub const MAX_SEARCH_RADIUS: u32 = 256;

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
