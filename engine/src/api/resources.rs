use super::*;
use crate::model::{self, EntityId, Resource as ResourceComponent};
use crate::profile;
use crate::storage::Storage;
use crate::tables::{JoinIterator, PositionTable};
use caolo_api::point::{Circle, Point};
use caolo_api::resources::{Mineral, Resource, Resources};
use rayon::prelude::*;

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

#[no_mangle]
pub fn _get_max_search_radius(_ctx: &mut Ctx) -> i32 {
    MAX_SEARCH_RADIUS as i32
}

/// Find available resources in the given range
/// `q` is the q or x parameter of the center of the circle
/// `r` is the r or y parameter of the center of the circle
/// results are sorted by their distance to the given point
#[no_mangle]
pub fn _find_resources_in_range(ctx: &mut Ctx, q: i32, r: i32, radius: i32, ptr: i32) -> i32 {
    profile!("_find_resources_in_range");
    let c = Circle {
        center: Point::new(q, r),
        radius: radius as u32,
    };
    debug!("_find_resources_in_range {:?}", c);

    if radius as u32 > MAX_SEARCH_RADIUS {
        error!("Radius of {} is too large", radius);
        return OperationResult::InvalidInput as i32;
    }

    let storage = unsafe { get_storage(ctx) };
    let positions = storage.entity_table::<model::PositionComponent>();

    let candidates = positions.get_entities_in_range(&c);
    let ids = candidates
        .iter()
        .map(|(id, _)| id)
        .cloned()
        .collect::<Vec<_>>();

    let resources = storage.entity_table::<model::Resource>();
    let resources = resources.get_by_ids(&ids);

    let energy = storage.entity_table::<model::EnergyComponent>();
    let mut result = JoinIterator::new(candidates.into_iter(), resources.into_iter())
        .filter_map(|(id, (pos, res))| match res {
            ResourceComponent::Mineral => {
                let energy = energy.get_by_id(&id).or_else(|| {
                    error!("Mineral {:?} has no energy", id);
                    None
                })?;

                let mineral = Mineral::new(id, pos.0, energy.energy, energy.energy_max);

                Some(Resource::Mineral(mineral))
            }
        })
        .collect::<Vec<_>>();

    result.par_sort_by_key(|r| r.position().hex_distance(c.center));

    let result = Resources::new(result);
    let data = result.serialize();
    let len = data.len();

    save_bytes_to_memory(ctx, ptr as usize, len, &data);

    debug!(
        "_find_resources_in_range written {} bytes, returns {:?}",
        len, len
    );
    len as i32
}
