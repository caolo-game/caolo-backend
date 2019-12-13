use super::*;
use crate::storage::TableId;
use rayon::prelude::*;
use std::ops::Add;

pub trait SpatialKey2d: TableId + Add<Output = Self> {
    /// Get axis 0 or 1
    fn get_axis(&self, axis: u8) -> i32;

    /// Construct a new key with given coordinates
    fn new(x: i32, y: i32) -> Self;

    /// Distance between two keys
    fn dist(&self, other: &Self) -> u32;

    /// Distance amoung given axis
    fn axis_dist(&self, other: &Self, axis: u8) -> u32 {
        (self.get_axis(axis) - other.get_axis(axis)).abs() as u32
    }
}

#[derive(Default, Debug)]
pub struct QuadtreeTable<Id, Row>
where
    Id: SpatialKey2d,
    Row: TableRow,
{
    median: Id,
    radius: u32,

    data: Vec<(Id, Row)>,
    capacity: usize,

    children: Option<Vec<Self>>,
}

impl<Id, Row> QuadtreeTable<Id, Row>
where
    Id: SpatialKey2d,
    Row: TableRow,
{
    pub fn new(median: Id, radius: u32, capacity: usize) -> Self {
        Self {
            median,
            radius,
            capacity,

            data: Vec::with_capacity(capacity),
            children: None,
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

    fn test_aabb_aabb(a: &Id, radiusa: u32, b: &Id, radiusb: u32) -> bool {
        for i in 0..2 {
            if a.axis_dist(b, i) as i64 > (radiusa as i64 + radiusb as i64) {
                return false;
            }
        }
        true
    }

    /// Returns wether the insertion succeeded
    pub fn insert(&mut self, (id, row): (Id, Row)) -> bool {
        if !Self::test_aabb_aabb(&id, 0, &self.median, self.radius) {
            println!(
                "med: {:?} rad: {:?} | point: {:?}",
                self.median, self.radius, id
            );
            return false;
        }
        if self.data.len() < self.capacity {
            self.data.push((id, row));
            return true;
        }
        if self.children.is_none() {
            self.split();
        }
        let ind = self.child_index(&id);
        self.children.as_mut().unwrap()[ind as usize].insert((id, row))
    }

    fn split(&mut self) {
        assert!(self.children.is_none(), "splitting node more than once!");

        let radius = (self.radius / 2 + 1) as i32;
        let mut children = Vec::with_capacity(4);
        for x in (-1..=1).step_by(2) {
            for y in (-1..=1).step_by(2) {
                children.push(Self::new(
                    self.median + Id::new(x * radius, y * radius),
                    radius as u32,
                    self.capacity,
                ));
            }
        }
        assert_eq!(
            children.len(),
            4,
            "Split produced an invalid number of children"
        );
        children.sort_by_key(|c| self.child_index(&c.median));
        self.children = Some(children);
    }

    pub fn child_index(&self, id: &Id) -> usize {
        let mut res = 0;
        for i in 0..2 {
            if self.median.get_axis(i) < id.get_axis(i) {
                res |= 1 << i;
            }
        }
        res
    }
}

impl<Id, Row> Table for QuadtreeTable<Id, Row>
where
    Id: SpatialKey2d + Send + Sync,
    Row: TableRow + Send + Sync,
{
    type Id = Id;
    type Row = Row;

    fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        if let Some((_, row)) = self
            .data
            .iter()
            .find(|(p, _)| p == id)
        {
            return Some(row);
        }
        if let Some(ref children) = self.children {
            let ind = self.child_index(id);
            if let Some(row) = children[ind].get_by_id(id) {
                return Some(row);
            }
        }
        None
    }

    fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        ids.into_par_iter()
            .filter_map(|id| self.get_by_id(id).map(|row| (*id, row)))
            .collect()
    }

    fn insert(&mut self, id: Id, row: Row) {
        assert!(self.insert((id, row)), "insertion failed");
    }

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
        if let Some(children) = self.children.as_mut() {
            if let Some(row) = children[ind].delete(id) {
                return Some(row);
            }
        }
        None
    }
}

impl PositionTable for QuadtreeTable<PositionComponent, EntityId> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        let mut res = Vec::new();
        self.find_by_range(
            &PositionComponent(vision.center),
            vision.radius * 3 / 2,
            &mut res,
        );
        res.into_iter()
            .filter(|(pos, _)| pos.0.hex_distance(vision.center) <= u64::from(vision.radius))
            .map(|(pos, id)| (*id, *pos))
            .collect()
    }

    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        let mut res = Vec::new();
        self.find_by_range(
            &PositionComponent(vision.center),
            vision.radius * 3 / 2,
            &mut res,
        );
        res.into_iter()
            .filter(|(pos, _)| pos.0.hex_distance(vision.center) <= u64::from(vision.radius))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caolo_api::point::Point;
    use rand::prelude::*;
    use test::Bencher;

    #[test]
    fn insertions() {
        let mut tree = QuadtreeTable::new(Point::default(), 128, 16);

        let r = tree.insert((Point::new(16, 32), 123i32));
        assert!(r);
        let r = tree.insert((Point::new(1600, 32), 123i32));
        assert!(!r);
    }

    #[test]
    fn test_range_query() {
        let mut rng = rand::thread_rng();

        let mut tree = QuadtreeTable::new(Point::default(), 128, 8);

        for i in 0..256 {
            let p = Point {
                x: rng.gen_range(-127, 128),
                y: rng.gen_range(-127, 128),
            };
            let inserted = tree.insert((p, i));
            assert!(inserted);
        }

        let mut res = Vec::new();
        tree.find_by_range(&Point::default(), 256, &mut res);

        assert_eq!(res.len(), 256);
    }

    #[test]
    fn get_by_id() {
        let mut rng = rand::thread_rng();

        let mut tree = QuadtreeTable::new(Point::default(), 128, 8);

        let mut points = Vec::with_capacity(256);

        for i in 0..8usize {
            let p = Point {
                x: rng.gen_range(-127, 128),
                y: rng.gen_range(-127, 128),
            };
            points.push((p, i));
        }

        for p in points.iter().cloned() {
            let inserted = tree.insert(p);
            assert!(inserted);
        }

        points.shuffle(&mut rng);

        println!("{:?}\n{:#?}", points, tree);

        for p in points {
            let found = tree.get_by_id(&p.0);
            assert_eq!(found, Some(&p.1));
        }
    }

    #[bench]
    fn bench_range_query(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = QuadtreeTable::new(Point::default(), 4000, 8);

        for i in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            let inserted = tree.insert((p, i));
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
            tree.find_by_range(&p, radius, &mut res);
            res.len()
        });
    }

    #[bench]
    fn bench_get_entities_in_range(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = QuadtreeTable::new(PositionComponent::default(), 4000, 16);

        for i in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            let p = PositionComponent(p);
            let inserted = tree.insert((p, EntityId(rng.gen())));
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
}
