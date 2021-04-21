use std::collections::{BinaryHeap, HashMap};

use crate::{
    components::{EntityComponent, TerrainComponent},
    indices::RoomPosition,
    prelude::{Axial, View},
    profile,
    tables::hex_grid::HexGrid,
};
use tracing::{debug, trace};

use super::{is_walkable, Node, PathFindingError};

/// Returns the remaining steps.
/// Uses the A* algorithm
pub fn find_path_in_room(
    from: Axial,
    to: Axial,
    distance: u32,
    (positions, terrain): (View<Axial, EntityComponent>, View<Axial, TerrainComponent>),
    max_steps: u32,
    path: &mut Vec<RoomPosition>,
) -> Result<u32, PathFindingError> {
    profile!("find_path_in_room");
    trace!("find_path_in_room from {:?} to {:?}", from, to);

    let current = from;
    let end = to;

    let mut remaining_steps = max_steps;

    let mut closed_set = HashMap::<Axial, Node>::with_capacity(remaining_steps as usize);
    let mut open_set = BinaryHeap::with_capacity(remaining_steps as usize);
    let mut open_set_visited = HexGrid::<bool>::new(terrain.bounds().radius as usize);

    let mut current = Node::new(current, current, current.hex_distance(end) as i32, 0);
    closed_set.insert(current.pos, current.clone());
    open_set.push(current.clone());

    while !open_set.is_empty() && remaining_steps > 0 {
        if current.pos.hex_distance(end) <= distance {
            // done
            // reconstruct path
            let mut current = current.pos;
            let end = from;
            while current != end {
                path.push(RoomPosition(current));
                current = closed_set[&current].parent;
            }
            debug!(
                "find_path_in_room succeeded, steps taken: {} remaining_steps: {}",
                max_steps - remaining_steps,
                remaining_steps,
            );
            return Ok(remaining_steps);
        }
        current = open_set.pop().unwrap();
        closed_set.insert(current.pos, current.clone());
        for point in &current.pos.hex_neighbours() {
            let point = *point;
            // Filter only the free neighbours
            // End may be in the either tables!
            if (point != end && (positions.contains_key(point))
                || open_set_visited.at(point).copied().unwrap_or(false)
                || !is_walkable(point, terrain))
                || closed_set.contains_key(&point)
            {
                continue;
            }
            open_set_visited[point] = true;
            let node = Node::new(
                point,
                current.pos,
                point.hex_distance(end) as i32,
                current.g_cost + 1,
            );
            open_set.push(node);
        }
        remaining_steps -= 1;
    }
    // failed

    debug!(
        "find_path_in_room failed, steps taken: {} remaining_steps: {}",
        max_steps - remaining_steps,
        remaining_steps
    );
    if remaining_steps > 0 {
        // we ran out of possible paths
        return Err(PathFindingError::Unreachable);
    }
    Err(PathFindingError::Timeout)
}
