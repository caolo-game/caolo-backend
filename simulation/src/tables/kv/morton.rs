//! Linear Quadtree.
//! # Contracts:
//! - Key axis must be in the interval [0, 2^16]
//! This is a severe restriction on the keys that can be used, however dense queries and
//! constructing from iterators is much faster than quadtrees.
//!
use super::*;
use crate::model::{components::EntityComponent, Point};
use rayon::prelude::*;
use std::convert::TryInto;
use std::marker::PhantomData;

use crate::profile;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct MortonKey(u32);

impl MortonKey {
    pub fn new(x: u16, y: u16) -> Self {
        Self(Self::morton2(x as u32, y as u32))
    }

    fn morton2(x: u32, y: u32) -> u32 {
        (Self::partition(x) + (Self::partition(y) << 1))
    }

    fn partition(mut n: u32) -> u32 {
        // n = ----------------fedcba9876543210 : Bits initially
        // n = --------fedcba98--------76543210 : After (1)
        // n = ----fedc----ba98----7654----3210 : After (2)
        // n = --fe--dc--ba--98--76--54--32--10 : After (3)
        // n = -f-e-d-c-b-a-9-8-7-6-5-4-3-2-1-0 : After (4)
        n = (n ^ (n << 8)) & 0x00ff00ff; // (1)
        n = (n ^ (n << 4)) & 0x0f0f0f0f; // (2)
        n = (n ^ (n << 2)) & 0x33333333; // (3)
        (n ^ (n << 1)) & 0x55555555 // (4)
    }
}

#[derive(Debug, Clone)]
struct Node {
    x: u16,
    y: u16,
    key: MortonKey,
    /// index of the corresponding Row
    ind: usize,
}

impl Node {
    pub fn new(x: u16, y: u16, ind: usize) -> Self {
        Self {
            x,
            y,
            ind,
            key: MortonKey::new(x, y),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct MortonTable<Id, Row>
where
    Id: SpatialKey2d,
    Row: TableRow,
{
    keys: Vec<Node>,
    values: Vec<Row>,

    _m: PhantomData<Id>,
}

unsafe impl<Id, Row> Send for MortonTable<Id, Row>
where
    Id: SpatialKey2d + Send,
    Row: TableRow + Send,
{
}

impl<Id, Row> MortonTable<Id, Row>
where
    Id: SpatialKey2d + Sync,
    Row: TableRow + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            values: vec![],
            keys: vec![],
            _m: Default::default(),
        }
    }

    pub fn from_iterator<It>(it: It) -> Self
    where
        It: Iterator<Item = (Id, Row)>,
    {
        let mut res = Self::new();
        res.extend(it);
        res
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.keys.clear();
    }

    pub fn extend<It>(&mut self, it: It)
    where
        It: Iterator<Item = (Id, Row)>,
    {
        for (id, value) in it {
            let [x, y] = id.as_array();
            self.keys.push(Node::new(
                x.try_into().expect("positive integer fitting into 16 bits"),
                y.try_into().expect("positive integer fitting into 16 bits"),
                self.keys.len(),
            ));
            self.values.push(value);
        }
        self.sort();
    }

    /// May trigger reordering of items, if applicable prefer `extend` and insert many keys at once.
    pub fn insert(&mut self, id: Id, row: Row) -> bool {
        let [x, y] = id.as_array();
        let x = x.try_into().expect("positive integer fitting into 16 bits");
        let y = y.try_into().expect("positive integer fitting into 16 bits");
        let ind = self
            .keys
            .binary_search_by_key(&MortonKey::new(x, y), |node| node.key)
            .unwrap_or_else(|i| i);
        self.keys.insert(ind, Node::new(x, y, self.values.len()));
        self.values.push(row);
        true
    }

    fn sort(&mut self) {
        self.keys.par_sort_by_key(|node| node.key);
    }

    /// Returns the first item with given id, if any
    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        profile!("get_by_id");

        let [x, y] = id.as_array();
        let x = x.try_into().ok()?;
        let y = y.try_into().ok()?;
        if let Ok(ind) = self
            .keys
            .binary_search_by_key(&MortonKey::new(x, y), |node| node.key)
        {
            Some(&self.values[self.keys[ind].ind])
        } else {
            None
        }
    }

    /// For each id returns the first item with given id, if any
    pub fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        profile!("get_by_ids");

        ids.into_par_iter()
            .filter_map(|id| self.get_by_id(id).map(|row| (*id, row)))
            .collect()
    }

    /// Find in AABB
    pub fn find_by_range<'a>(&'a self, center: &Id, radius: u32, out: &mut Vec<(Id, &'a Row)>) {
        profile!("find_by_range");

        let r = radius as i32 / 2 + 1;
        let min = *center + Id::new(-r, -r);
        let max = *center + Id::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);

        for i in min..max {
            let node = &self.keys[i];
            let id = Id::new(node.x as i32, node.y as i32);
            if center.dist(&id) < radius {
                out.push((id, &self.values[node.ind]));
            }
        }
    }

    /// Find in AABB
    pub fn count_in_range<'a>(&'a self, center: &Id, radius: u32) -> u32 {
        profile!("count_in_range");

        let r = radius as i32 / 2 + 1;
        let min = *center + Id::new(-r, -r);
        let max = *center + Id::new(r, r);

        let [min, max] = self.morton_min_max(&min, &max);

        let mut count = 0;
        for i in min..max {
            let node = &self.keys[i];
            let id = Id::new(node.x as i32, node.y as i32);
            if center.dist(&id) < radius {
                count += 1;
            }
        }
        count
    }

    /// Turn AABB min-max to from-to indices
    fn morton_min_max(&self, min: &Id, max: &Id) -> [usize; 2] {
        let [minx, miny] = min.as_array();
        let [maxx, maxy] = max.as_array();

        let minx = minx
            .max(0)
            .try_into()
            .expect("positive integer fitting into 16 bits");
        let miny = miny
            .max(0)
            .try_into()
            .expect("positive integer fitting into 16 bits");

        let maxx = maxx
            .max(0)
            .try_into()
            .expect("positive integer fitting into 16 bits");
        let maxy = maxy
            .max(0)
            .try_into()
            .expect("positive integer fitting into 16 bits");

        // calculate the range we have to check
        let min = MortonKey::new(minx, miny);
        let max = MortonKey::new(maxx, maxy);

        // find the index for the min/max hash, or their place
        let min: usize = self
            .keys
            .binary_search_by_key(&min, |node| node.key)
            .unwrap_or_else(|i| i);
        let max: usize = self
            .keys
            .binary_search_by_key(&max, |node| node.key)
            .unwrap_or_else(|i| i);

        [min, max]
    }

    /// Return wether point is within the bounds of this node
    pub fn intersects(&self, point: &Id) -> bool {
        let [x, y] = point.as_array();
        // at most 16 bits long non-negative integers
        x >= 0 && y >= 0 && (x & 0x0000ffff) == x && (y & 0x0000ffff) == y
    }
}

impl<Id, Row> Table for MortonTable<Id, Row>
where
    Id: SpatialKey2d + Send + Sync,
    Row: TableRow + Send + Sync,
{
    type Id = Id;
    type Row = Row;

    fn delete(&mut self, id: &Id) -> Option<Row> {
        profile!("delete");

        let [x, y] = id.as_array();
        let x = x.try_into().ok()?;
        let y = y.try_into().ok()?;
        let id = MortonKey::new(x, y);
        match self.keys.binary_search_by_key(&id, |node| node.key) {
            Err(_) => None,
            Ok(ind) => {
                // swap with the last, reassign the corresponding key
                //
                let i = self.keys[ind].ind;
                let last = self.values.len() - 1;

                self.values.swap(i, last);

                for n in self.keys.iter_mut() {
                    if n.ind == last {
                        n.ind = i;
                        break;
                    }
                }

                self.keys.remove(ind);
                Some(self.values.remove(last))
            }
        }
    }
}

impl PositionTable for MortonTable<Point, EntityComponent> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        profile!("get_entities_in_range");

        let mut res = Vec::new();
        self.find_by_range(&vision.center, vision.radius * 3 / 2, &mut res);
        res.into_iter()
            .filter(|(pos, _)| pos.hex_distance(vision.center) <= u64::from(vision.radius))
            .map(|(pos, id)| (id.0, PositionComponent(pos)))
            .collect()
    }

    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        profile!("count_entities_in_range");

        self.count_in_range(&vision.center, vision.radius * 3 / 2) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caolo_api::point::Point;
    use rand::prelude::*;
    use std::collections::HashMap;
    use test::Bencher;

    #[test]
    fn insertions() {
        let mut tree = MortonTable::new();

        let r = tree.insert(Point::new(16, 32), 123i32);
        assert!(r);
    }

    #[test]
    fn test_range_query_all() {
        let mut rng = rand::thread_rng();

        let mut tree = MortonTable::new();

        for i in 0..256 {
            let p = Point {
                x: rng.gen_range(0, 128),
                y: rng.gen_range(0, 128),
            };
            let inserted = tree.insert(p, i);
            assert!(inserted);
        }

        let mut res = Vec::new();
        tree.find_by_range(&Point::default(), 256, &mut res);

        assert_eq!(res.len(), 256);
    }

    #[test]
    fn get_by_id() {
        let mut rng = rand::thread_rng();

        let mut tree = MortonTable::<Point, usize>::new();

        let mut points = HashMap::with_capacity(64);

        for i in 0..64 {
            let p = Point {
                x: rng.gen_range(0, 128),
                y: rng.gen_range(0, 128),
            };
            points.insert(p, i);
        }

        for (p, e) in points.iter() {
            let inserted = tree.insert(p.clone(), *e);
            assert!(inserted);
        }

        let mut points: Vec<_> = points.into_iter().collect();

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

        let mut tree = MortonTable::new();

        for i in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            let inserted = tree.insert(p, i);
            assert!(inserted);
        }

        let mut res = Vec::with_capacity(512);
        let radius = 512;

        b.iter(|| {
            let tree = &tree;
            res.clear();
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            tree.find_by_range(&p, radius, &mut res);
            res.len()
        });
    }

    #[bench]
    fn bench_get_entities_in_range_sparse(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = MortonTable::new();

        for _ in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            let inserted = tree.insert(p, EntityComponent(EntityId(rng.gen())));
            assert!(inserted);
        }

        let radius = 512;

        b.iter(|| {
            let tree = &tree;
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            tree.get_entities_in_range(&Circle { center: p, radius })
        });
    }

    #[bench]
    fn bench_get_entities_in_range_dense(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = MortonTable::new();

        for _ in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(0, 200 * 2),
                y: rng.gen_range(0, 200 * 2),
            };
            let inserted = tree.insert(p, EntityComponent(EntityId(rng.gen())));
            assert!(inserted);
        }

        let radius = 50;

        b.iter(|| {
            let tree = &tree;
            let p = Point {
                x: rng.gen_range(0, 200 * 2),
                y: rng.gen_range(0, 200 * 2),
            };
            tree.get_entities_in_range(&Circle { center: p, radius })
        });
    }

    #[bench]
    fn make_morton_tree(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let tree = MortonTable::from_iterator((0..(1 << 15)).map(|_| {
                (
                    Point {
                        x: rng.gen_range(0, 3900 * 2),
                        y: rng.gen_range(0, 3900 * 2),
                    },
                    rng.next_u32(),
                )
            }));
            tree
        });
    }

    #[bench]
    fn rebuild_morton_tree(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = MortonTable::from_iterator((0..(1 << 15)).map(|_| {
            (
                Point {
                    x: rng.gen_range(0, 3900 * 2),
                    y: rng.gen_range(0, 3900 * 2),
                },
                rng.next_u32(),
            )
        }));

        b.iter(|| {
            tree.clear();

            tree.extend((0..(1 << 15)).map(|_| {
                (
                    Point {
                        x: rng.gen_range(0, 3900 * 2),
                        y: rng.gen_range(0, 3900 * 2),
                    },
                    rng.next_u32(),
                )
            }));
        })
    }

    #[bench]
    fn bench_get_by_id_rand(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let len = 1 << 16;
        let tree = MortonTable::from_iterator((0..len).map(|_| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            (pos, rng.next_u32())
        }));

        b.iter(|| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            tree.get_by_id(&pos)
        });
    }

    #[bench]
    fn bench_get_by_id_in_tree(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let len = 1 << 16;
        let mut points = Vec::with_capacity(len);
        let tree = MortonTable::from_iterator((0..len).map(|_| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            points.push(pos.clone());
            (pos, rng.next_u32())
        }));

        b.iter(|| {
            let i = rng.gen_range(0, points.len());
            let pos = &points[i];
            tree.get_by_id(pos)
        });
    }

    #[bench]
    fn bench_get_by_id_in_hashmap(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let len = 1 << 16;
        let mut points = Vec::with_capacity(len);
        let mut tree = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            points.push(pos.clone());
            tree.insert(pos, rng.next_u32());
        }

        b.iter(|| {
            let i = rng.gen_range(0, points.len());
            let pos = &points[i];
            tree.get(pos)
        });
    }

    #[bench]
    fn bench_get_by_id_rand_in_hashmap(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let len = 1 << 16;

        let mut tree = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            tree.insert(pos, rng.next_u32());
        }

        b.iter(|| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            tree.get(&pos)
        });
    }

    #[bench]
    fn bench_get_entities_in_range_dense_in_hashmap(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut tree = std::collections::HashMap::new();

        for _ in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(0, 200 * 2),
                y: rng.gen_range(0, 200 * 2),
            };
            tree.insert(p, EntityComponent(EntityId(rng.gen())));
        }

        let radius = 50;

        let mut v = Vec::with_capacity(512);
        b.iter(|| {
            let tree = &tree;
            let x = rng.gen_range(0, 200 * 2);
            let y = rng.gen_range(0, 200 * 2);
            v.clear();
            for x in x - radius..x + radius {
                for y in y - radius..y + radius {
                    let p = Point { x, y };
                    if let Some(val) = tree.get(&p) {
                        v.push((p, val));
                    }
                }
            }
            v.len()
        });
    }
}
