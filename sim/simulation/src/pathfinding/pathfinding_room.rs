use std::collections::BinaryHeap;

use crate::{
    components::{EntityComponent, TerrainComponent},
    indices::RoomPosition,
    prelude::{Axial, Hexagon, View},
    profile,
    tables::hex_grid::HexGrid,
};
use tracing::{debug, trace};

use super::{is_walkable, Node, PathFindingError};

const VISITED_FROM: u8 = 1 << 0;
const VISITED_TO: u8 = 1 << 1;
type Bounds = [Axial; 2];

/// The goal of the pathfinder to approach `end` at a distance of `distance`.
///
/// So we'll initialize a ring of nodes with the center `end` and radius `distance`.
fn init_end(
    [begin, end]: Bounds,
    distance: u32,
    entities: View<Axial, EntityComponent>,
    terrain: View<Axial, TerrainComponent>,
    open_set: &mut BinaryHeap<Node>,
    visited: &mut HexGrid<u8>,
    closed_set: &mut HexGrid<Node>,
) {
    if distance == 0 {
        // `iter_edge` returns empty if radius is 0 so push the pos here
        let pos = end;
        if let Some(v) = visited.at_mut(pos) {
            *v |= VISITED_TO;
            let n = Node::new(pos, pos, pos.hex_distance(begin) as i32, 0);
            open_set.push(n.clone());
            closed_set[pos] = n;
        }
    } else {
        let bounds = Hexagon::new(end, distance as i32);
        for pos in bounds.iter_edge().filter(|pos| {
            terrain
                .at(*pos)
                .map(|TerrainComponent(t)| t.is_walkable())
                .unwrap_or(false)
                && !entities.contains_key(*pos)
        }) {
            debug_assert_eq!(pos.hex_distance(end), distance);
            if let Some(v) = visited.at_mut(pos) {
                *v |= VISITED_TO;
                let n = Node::new(pos, pos, pos.hex_distance(begin) as i32, 0);
                open_set.push(n.clone());
                closed_set[pos] = n;
            }
        }
    }
}

fn reconstruct_path(
    current: Axial,
    start: Axial,
    end: Axial,
    distance: u32,
    path: &mut Vec<RoomPosition>,
    closed_set_f: &HexGrid<Node>,
    closed_set_t: &HexGrid<Node>,
) {
    // reconstruct 'to'
    //
    // parents move towards `end`
    {
        let i = path.len();
        // copy current
        let mut current = current;
        // 'current' will be pushed by the second loop
        while current.hex_distance(end) > distance {
            current = closed_set_t[current].parent;
            path.push(RoomPosition(current));
        }
        path[i..].reverse();
    }

    // reconstruct 'from'
    //
    // parents move towards `start`
    let mut current = current;
    while current != start {
        path.push(RoomPosition(current));
        current = closed_set_f[current].parent;
    }
}

/// Returns the remaining steps.
///
/// The algorithm is a two-way A*, where we start A* from both the `from` and the `to` points and
/// exit when they meet.
/// This should reduce the size of the graph we need to traverse in the general case.
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

    if from.hex_distance(to) <= distance {
        return Ok(max_steps);
    }

    let end = to;

    let mut remaining_steps = max_steps;

    let room_radius = terrain.bounds().radius;
    debug_assert!(room_radius >= 0);

    let mut closed_set_f = HexGrid::<Node>::new(room_radius as usize);
    let mut open_set_f = BinaryHeap::with_capacity(remaining_steps as usize);

    let mut closed_set_t = HexGrid::<Node>::new(room_radius as usize);
    let mut open_set_t = BinaryHeap::with_capacity(remaining_steps as usize);

    let mut open_set_visited = HexGrid::<u8>::new(room_radius as usize);

    init_end(
        [from, end],
        distance,
        positions,
        terrain,
        &mut open_set_t,
        &mut open_set_visited,
        &mut closed_set_t,
    );

    let mut current_f = Node::new(from, from, from.hex_distance(end) as i32, 0);
    closed_set_f
        .insert(current_f.pos, current_f.clone())
        .unwrap();
    open_set_f.push(current_f.clone());

    while !open_set_f.is_empty() && !open_set_t.is_empty() && remaining_steps > 0 {
        // if we find this position in the other set
        if closed_set_t[current_f.pos].g_cost != 0 {
            reconstruct_path(
                current_f.pos,
                from,
                to,
                distance,
                path,
                &closed_set_f,
                &closed_set_t,
            );
            debug!(
                "find_path_in_room succeeded, steps taken: {} remaining_steps: {}",
                max_steps - remaining_steps,
                remaining_steps,
            );
            return Ok(remaining_steps);
        }
        // step `from`
        {
            current_f = open_set_f.pop().unwrap();
            closed_set_f
                .insert(current_f.pos, current_f.clone())
                .unwrap();
            for point in &current_f.pos.hex_neighbours() {
                let point = *point;
                if open_set_visited.at(point).copied().unwrap_or(VISITED_FROM) & VISITED_FROM != 0
                    || positions.contains_key(point)
                    || !is_walkable(point, terrain)
                    || closed_set_f
                        .at(point)
                        .map(|node| node.g_cost != 0)
                        .unwrap_or(false)
                {
                    continue;
                }
                open_set_visited[point] |= VISITED_FROM;
                let node = Node::new(
                    point,
                    current_f.pos,
                    point.hex_distance(end) as i32,
                    current_f.g_cost + 1,
                );
                open_set_f.push(node);
            }
        }
        // step `to`
        {
            let current_t = open_set_t.pop().unwrap();
            closed_set_t
                .insert(current_t.pos, current_t.clone())
                .unwrap();
            // if we find this position in the other set
            if closed_set_f[current_t.pos].g_cost != 0 {
                reconstruct_path(
                    current_t.pos,
                    from,
                    to,
                    distance,
                    path,
                    &closed_set_f,
                    &closed_set_t,
                );
                debug!(
                    "find_path_in_room succeeded, steps taken: {} remaining_steps: {}",
                    max_steps - remaining_steps,
                    remaining_steps,
                );
                return Ok(remaining_steps);
            }
            for point in &current_t.pos.hex_neighbours() {
                let point = *point;
                if point.hex_distance(end) <= distance
                    || open_set_visited.at(point).copied().unwrap_or(VISITED_TO) & VISITED_TO != 0
                    || !is_walkable(point, terrain)
                    || positions.contains_key(point)
                    || closed_set_t
                        .at(point)
                        .map(|node| node.g_cost != 0)
                        .unwrap_or(false)
                {
                    continue;
                }
                open_set_visited[point] |= VISITED_TO;
                let node = Node::new(
                    point,
                    current_t.pos,
                    point.hex_distance(from) as i32,
                    current_t.g_cost + 1,
                );
                open_set_t.push(node);
            }
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
