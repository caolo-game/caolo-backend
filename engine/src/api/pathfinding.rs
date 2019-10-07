use super::*;
use crate::model;
use crate::profile;
use crate::systems::pathfinding::find_path;
use caolo_api::point::Point;

const MAX_STEPS: u32 = 64;

/// Return the maximum number of steps a path may contain
#[no_mangle]
pub fn _get_max_path_length(_ctx: &mut Ctx) -> i32 {
    MAX_STEPS as i32
}

/// Return an OperationResult and the length of the Path object on success
#[no_mangle]
pub fn _find_path(ctx: &mut Ctx, fromx: i32, fromy: i32, tox: i32, toy: i32, outptr: i32) -> i32 {
    profile!("_find_path");

    let storage = unsafe { get_storage(ctx) };
    let positions = storage.entity_table::<model::PositionComponent>();
    let terrain = storage.point_table::<model::TileTerrainType>();

    match find_path(
        Point::new(fromx, fromy),
        Point::new(tox, toy),
        positions,
        terrain,
        MAX_STEPS,
    ) {
        Ok(path) => {
            let path = caolo_api::pathfinding::Path { path };
            let data = path.serialize();
            let len = data.len();
            save_bytes_to_memory(ctx, outptr as usize, len, &data);
            len as i32
        }
        Err(e) => {
            debug!(
                "Failed to find path from {:?} to {:?} {:?}",
                Point::new(fromx, fromy),
                Point::new(tox, toy),
                e
            );
            OperationResult::OperationFailed as i32
        }
    }
}
