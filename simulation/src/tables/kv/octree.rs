//! Octree based tables to query three dimensional data
//!
use super::*;
use crate::model::{components::EntityComponent, Point3};
use crate::storage::TableId;
use arrayvec::ArrayVec;
use rayon::prelude::*;
use std::ops::Add;

pub trait SpacialKey3d: TableId + Add<Output = Self> {
    /// Get axis 0 or 1 or 2
    fn get_axis(&self, axis: u8) -> i32;

    /// Construct a new key with given coordinates
    fn new(x: i32, y: i32, z: i32) -> Self;

    /// Distance between two keys
    fn dist(&self, other: &Self) -> u32;

    /// Distance among given axis. Used for separating axis tests to reduce query times when only
    /// one axis is considered..
    fn axis_dist(&self, other: &Self, axis: u8) -> u32 {
        (self.get_axis(axis) - other.get_axis(axis)).abs() as u32
    }
}

/// 2D spatial storage.
/// Keys are not unique! If you're unsure wether a key appears multiple times use range querying
/// methods.
#[derive(Default, Debug)]
pub struct OctreeTable<Id, Row>
where
    Id: SpacialKey3d,
    Row: TableRow,
{
    median: Id,
    radius: u32,

    data: ArrayVec<[(Id, Row); 8]>,

    children: Option<Vec<Self>>,
}

impl<Id, Row> OctreeTable<Id, Row>
where
    Id: SpacialKey3d + Sync,
    Row: TableRow + Send + Sync,
{
    pub fn new(median: Id, radius: u32) -> Self {
        Self {
            median,
            radius,

            data: ArrayVec::new(),
            children: None,
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                child.clear();
            }
        }
    }

    /// Find in AABB
    pub fn find_by_range<'a>(&'a self, center: &Id, radius: u32, out: &mut Vec<&'a (Id, Row)>) {
        if !Self::test_aabb_aabb(&self.median, self.radius, center, radius) {
            return;
        }
        out.extend(self.data.iter().filter(|(p, _)| p.dist(center) < radius));
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.find_by_range(center, radius, out);
            }
        }
    }

    /// Return the bounds of this node as an AABB (min, max).
    pub fn bounds(&self) -> (Id, Id) {
        let radius = self.radius as i32;
        let x = self.median.get_axis(0);
        let y = self.median.get_axis(1);
        let z = self.median.get_axis(2);

        (
            Id::new(x - radius, y - radius, z - radius),
            Id::new(x + radius, y + radius, z + radius),
        )
    }

    /// Return wether point is within the bounds of this node
    pub fn intersects(&self, point: &Id) -> bool {
        let x = point.get_axis(0);
        let y = point.get_axis(1);

        let mx = self.median.get_axis(0);
        let my = self.median.get_axis(1);

        let rad = self.radius as i32;

        mx - rad <= x && my - rad <= y && x <= mx + rad && y <= my + rad
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        if let Some((_, row)) = self.data.iter().find(|(p, _)| p == id) {
            return Some(row);
        }
        if let Some(ref children) = self.children {
            let ind = self.child_index(id);
            match children[ind].get_by_id(id) {
                row @ Some(_) => return row,
                None => {}
            }
        }
        None
    }

    pub fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        ids.into_par_iter()
            .filter_map(|id| self.get_by_id(id).map(|row| (*id, row)))
            .collect()
    }

    pub fn insert(&mut self, id: Id, row: Row) -> bool {
        if !Self::test_aabb_aabb(&id, 0, &self.median, self.radius) {
            debug!(
                "Insertion out of bounds med: {:?} rad: {:?} | point: {:?}",
                self.median, self.radius, id
            );
            return false;
        }
        if self.data.len() < self.data.capacity() {
            self.data.push((id, row));
            return true;
        }
        let ind = self.child_index(&id);
        match self.children.as_mut() {
            Some(children) => {
                children[ind as usize].insert(id, row);
            }
            None => {
                self.split()[ind as usize].insert(id, row);
            }
        }
        true
    }

    fn test_aabb_aabb(a: &Id, radiusa: u32, b: &Id, radiusb: u32) -> bool {
        let rad = radiusa as i64 + radiusb as i64;
        if a.axis_dist(b, 0) as i64 > rad
            || a.axis_dist(b, 1) as i64 > rad
            || a.axis_dist(b, 2) as i64 > rad
        {
            return false;
        }
        true
    }

    fn split(&mut self) -> &mut Vec<Self> {
        assert!(self.children.is_none(), "splitting node more than once!");

        let radius = (self.radius / 2 + 1) as i32;
        let mut children = Vec::with_capacity(4);
        for x in (-1..=1).step_by(2) {
            for y in (-1..=1).step_by(2) {
                for z in (-1..=1).step_by(2) {
                    children.push(Self::new(
                        self.median + Id::new(x * radius, y * radius, z * radius),
                        radius as u32,
                    ));
                }
            }
        }
        assert_eq!(
            children.len(),
            8,
            "Split produced an invalid number of children"
        );
        children.sort_by_key(|c| self.child_index(&c.median));
        self.children = Some(children);
        self.children.as_mut().unwrap()
    }

    fn child_index(&self, id: &Id) -> usize {
        let mut res = 0;
        for i in 0..3 {
            if self.median.get_axis(i) < id.get_axis(i) {
                res |= 1 << i;
            }
        }
        res
    }
}

impl<Id, Row> Table for OctreeTable<Id, Row>
where
    Id: SpacialKey3d + Send + Sync,
    Row: TableRow + Send + Sync,
{
    type Id = Id;
    type Row = Row;

    fn delete(&mut self, id: &Id) -> Option<Row> {
        if let Some((i, row)) = self
            .data
            .iter()
            .enumerate()
            .find(|(_, (p, _))| Self::test_aabb_aabb(p, 0, id, 0))
            .map(|(i, (_, row))| (i, row.clone()))
        {
            self.data.remove(i);
            return Some(row);
        }
        let ind = self.child_index(id);
        self.children
            .as_mut()
            .and_then(|children| children[ind].delete(id))
    }
}

impl PositionTable for OctreeTable<Point3, EntityComponent> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        let mut res = Vec::new();
        let center = Point3::hex_axial_to_cube(vision.center);
        self.find_by_range(&center, vision.radius * 3 / 2, &mut res);
        res.into_iter()
            .filter(|(pos, _)| pos.hex_distance(center) <= u64::from(vision.radius))
            .map(|(pos, id)| (id.0, PositionComponent(pos.into_axial())))
            .collect()
    }

    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        let mut res = Vec::new();
        let center = Point3::hex_axial_to_cube(vision.center);
        self.find_by_range(&center, vision.radius * 3 / 2, &mut res);
        res.into_iter()
            .filter(|(pos, _)| pos.hex_distance(center) <= u64::from(vision.radius))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Circle, Point, Point3};
    use rand::prelude::*;
    use test::Bencher;

    #[test]
    fn insertions() {
        let mut tree = OctreeTable::new(Point3::default(), 128);

        let r = tree.insert(Point3::new(16, 32, 16), 123i32);
        assert!(r);
        let r = tree.insert(Point3::new(1600, 32, 16), 123i32);
        assert!(!r);
    }

    #[test]
    fn test_range_query_all() {
        let mut rng = rand::thread_rng();

        let mut tree = OctreeTable::new(Point3::default(), 128);

        for i in 0..256 {
            let p = Point3 {
                x: rng.gen_range(-127, 128),
                y: rng.gen_range(-127, 128),
                z: rng.gen_range(-127, 128),
            };
            let inserted = tree.insert(p, i);
            assert!(inserted);
        }

        let mut res = Vec::new();
        tree.find_by_range(&Point3::default(), 256, &mut res);

        assert_eq!(res.len(), 256);
    }

    #[test]
    fn get_by_id() {
        let mut rng = rand::thread_rng();

        let mut tree = OctreeTable::new(Point3::default(), 128);

        let mut points = Vec::with_capacity(256);

        for i in 0..256usize {
            let p = Point3 {
                x: rng.gen_range(-127, 128),
                y: rng.gen_range(-127, 128),
                z: rng.gen_range(-127, 128),
            };
            points.push((p, i));
        }

        for (p, e) in points.iter().cloned() {
            let inserted = tree.insert(p, e);
            assert!(inserted);
        }

        points.shuffle(&mut rng);

        println!("{:?}\n{:#?}", points, tree);

        for p in points {
            let found = tree.get_by_id(&p.0);
            assert_eq!(found, Some(&p.1));
        }

        let mut fake = Vec::with_capacity(256);
        for i in 0..256usize {
            let p = Point3 {
                x: rng.gen_range(128, 256),
                y: rng.gen_range(128, 256),
                z: rng.gen_range(128, 256),
            };
            fake.push((p, i));
        }

        for p in fake {
            let found = tree.get_by_id(&p.0);
            assert!(found.is_none());
        }
    }

    #[bench]
    fn bench_range_query(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = OctreeTable::new(Point3::default(), 8000);

        for i in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            let inserted = tree.insert(Point3::hex_axial_to_cube(p), i);
            assert!(inserted);
        }

        let mut res = Vec::with_capacity(512);
        let radius = 512;

        b.iter(|| {
            let tree = &tree;
            res.clear();
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            tree.find_by_range(&Point3::hex_axial_to_cube(p), radius, &mut res);
            res.len()
        });
    }

    #[bench]
    fn bench_get_entities_in_range(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = OctreeTable::new(Point3::default(), 8000);

        for _ in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            let inserted = tree.insert(
                Point3::hex_axial_to_cube(p),
                EntityComponent(EntityId(rng.gen())),
            );
            assert!(inserted);
        }

        let radius = 512;

        b.iter(|| {
            let tree = &tree;
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            tree.get_entities_in_range(&Circle { center: p, radius })
        });
    }

    #[bench]
    fn make_octree(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let mut tree = OctreeTable::new(Point3::default(), 4000);

            for _ in 0..(1 << 15) {
                let p = Point3 {
                    x: rng.gen_range(-3900, 3900),
                    y: rng.gen_range(-3900, 3900),
                    z: rng.gen_range(-3900, 3900),
                };
                let inserted = tree.insert(p, EntityComponent(EntityId(rng.gen())));
                assert!(inserted);
            }
        })
    }

    #[bench]
    fn rebuild_octree(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = OctreeTable::new(Point3::default(), 4000);

        for _ in 0..(1 << 15) {
            let p = Point3 {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
                z: rng.gen_range(-3900, 3900),
            };
            let inserted = tree.insert(p, EntityComponent(EntityId(rng.gen())));
            assert!(inserted);
        }

        b.iter(|| {
            tree.clear();

            for _ in 0..(1 << 15) {
                let p = Point3 {
                    x: rng.gen_range(-3900, 3900),
                    y: rng.gen_range(-3900, 3900),
                    z: rng.gen_range(-3900, 3900),
                };
                let inserted = tree.insert(p, EntityComponent(EntityId(rng.gen())));
                assert!(inserted);
            }
        })
    }
}
