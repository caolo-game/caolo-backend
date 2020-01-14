use crate::model::{EntityComponent, TerrainComponent, TileTerrainType};
use crate::tables::PositionTable;
use caolo_api::point::{Circle, Point};

use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Node {
    pub pos: Point,
    pub parent: Point,
    pub h: i32,
    pub g: i32,
}

impl Node {
    pub fn new(pos: Point, parent: Point, h: i32, g: i32) -> Self {
        Self { parent, h, g, pos }
    }

    pub fn f(&self) -> i32 {
        self.h + self.g
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathFindingError {
    NotFound,
    Unreachable,
}

/// Find path from `from` to `to`. Will append the resulting path to the `path` output vector.
pub fn find_path(
    from: Point,
    to: Point,
    positions: &<EntityComponent as crate::tables::Component<Point>>::Table,
    terrain: &<TerrainComponent as crate::tables::Component<Point>>::Table,
    mut max_iterations: u32,
    path: &mut Vec<Point>,
) -> Result<(), PathFindingError> {
    let center = (from + to) / 2;
    let circle = Circle {
        radius: from.hex_distance(center) as u32,
        center,
    };

    let mut terrain_tiles = Vec::with_capacity((circle.radius * circle.radius) as usize);
    terrain.find_by_range(&center, circle.radius, &mut terrain_tiles);

    let obsticles = positions
        .get_entities_in_range(&circle)
        .into_iter()
        .map(|(_, pos)| pos.0)
        .chain(terrain_tiles.iter().filter_map(|(p, t)| match t.0 {
            TileTerrainType::Wall => Some(*p),
            TileTerrainType::Empty => None,
        }))
        .collect::<HashSet<_>>();

    let current = from;
    let end = to;

    let mut closed_set = HashMap::<Point, Node>::with_capacity(circle.radius as usize * 2);
    let mut open_set = BTreeSet::new();

    let mut current = Node::new(current, current, current.hex_distance(end) as i32, 0);
    closed_set.insert(current.pos, current.clone());
    open_set.insert(current.clone());

    while current.pos != end && !open_set.is_empty() && max_iterations > 0 {
        current = open_set.iter().min_by_key(|node| node.f()).unwrap().clone();
        open_set.remove(&current);
        closed_set.insert(current.pos, current.clone());
        current
            .pos
            .neighbours()
            .iter()
            .filter(|p| {
                let res = positions.intersects(&p);
                debug_assert!(
                    terrain.intersects(&p) == res,
                    "if p intersects positions it must also intersect terrain!"
                );
                res
            })
            .filter(|p| {
                let is_inside = circle.is_inside(**p);

                (is_inside && !obsticles.contains(p))
                    || (positions.get_by_id(*p).is_none()
                        && terrain.get_by_id(*p).map(|x| &x.0) != Some(&TileTerrainType::Wall))
                    || **p == end // End may be in the positions table!
            })
            .for_each(|point| {
                let node = Node::new(
                    *point,
                    current.pos,
                    point.hex_distance(end) as i32,
                    current.g + 1,
                );
                if !open_set.contains(&node) && !closed_set.contains_key(point) {
                    open_set.insert(node);
                }
                if let Some(node) = closed_set.get_mut(point) {
                    if current.g + 1 < node.g {
                        node.g = current.g + 1;
                        node.parent = current.pos;
                    }
                }
            });
        max_iterations -= 1;
    }

    if current.pos != end {
        if max_iterations > 0 {
            Err(PathFindingError::Unreachable)?;
        }
        Err(PathFindingError::NotFound)?;
    }

    // reconstruct path
    let mut current = end;
    let end = from;
    let from = path.len();
    while current != end {
        path.push(current);
        current = closed_set[&current].parent;
    }
    // path is reconstructed from the end backwards, so fix the order of points
    path[from..].reverse();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EntityComponent, EntityId};
    use crate::tables::MortonTable;

    #[test]
    fn test_simple_wall() {
        let from = Point::new(0, 2);
        let to = Point::new(5, 2);

        let positions = MortonTable::new();
        let mut terrain = MortonTable::new();
        for i in 0..=5 {
            assert!(terrain.insert(Point::new(2, i), TerrainComponent(TileTerrainType::Wall)));
        }

        let mut path = vec![];
        find_path(from, to, &positions, &terrain, 512, &mut path).expect("Path finding failed");

        let mut current = from;
        for point in path.iter() {
            assert_eq!(point.hex_distance(current), 1);
            if point.x == 2 {
                assert!(point.y.abs() > 5, "{:?}", point);
            }
            current = *point;
        }
        assert_eq!(current, to);
    }

    #[test]
    fn test_simple() {
        let from = Point::new(17, 6);
        let to = Point::new(7, 16);

        let mut positions = MortonTable::new();
        let terrain = MortonTable::new();

        positions.insert(from, EntityComponent(EntityId(0)));
        positions.insert(to, EntityComponent(EntityId(1)));

        let mut path = vec![];
        find_path(from, to, &positions, &terrain, 512, &mut path).expect("Path finding failed");

        let mut current = from;
        for point in path.iter() {
            assert_eq!(point.hex_distance(current), 1);
            if point.x == 2 {
                assert!(point.y.abs() > 5, "{:?}", point);
            }
            current = *point;
        }
        assert_eq!(current, to);
    }
}
