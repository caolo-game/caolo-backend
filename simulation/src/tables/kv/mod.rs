//! Key-Value storages
//!
mod btree;
mod morton;
mod vector;

pub use btree::*;
pub use morton::*;
pub use vector::*;

use super::*;
use crate::storage::TableId;
use std::ops::Add;

pub trait SpatialKey2d: TableId + Add<Output = Self> {
    fn as_array(&self) -> [i32; 2];

    /// Get axis 0 or 1
    fn get_axis(&self, axis: u8) -> i32;

    /// Construct a new key with given coordinates
    fn new(x: i32, y: i32) -> Self;

    /// Distance between two keys
    fn dist(&self, other: &Self) -> u32;

    /// Distance among given axis. Used for separating axis tests to reduce query times when only
    /// one axis is considered..
    fn axis_dist(&self, other: &Self, axis: u8) -> u32 {
        (self.get_axis(axis) - other.get_axis(axis)).abs() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EntityId;
    use rand::Rng;
    use serde_derive::{Deserialize, Serialize};
    use test::{black_box, Bencher};

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    struct LargeComponent {
        _a: [u8; 10],
        _b: [u8; 10],
        _c: [u8; 10],
        _d: [u8; 10],
        _e: [u8; 10],
        _f: [u8; 10],
    }

    #[bench]
    fn join_vec_btree_2pow15(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut bt = BTreeTable::<EntityId, LargeComponent>::new();
        let mut ve = VecTable::<EntityId, LargeComponent>::new();
        for _ in 0..1 << 15 {
            let mut res = false;
            let mut id = EntityId::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 16));
                res = bt.insert_or_update(id, LargeComponent::default());
            }
            ve.insert_or_update(id, LargeComponent::default());
        }
        b.iter(move || {
            let it = JoinIterator::new(ve.iter(), bt.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_btree_vec_2pow15(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut bt = BTreeTable::<EntityId, LargeComponent>::new();
        let mut ve = VecTable::<EntityId, LargeComponent>::new();
        for _ in 0..1 << 15 {
            let mut res = false;
            let mut id = EntityId::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 16));
                res = bt.insert_or_update(id, LargeComponent::default());
            }
            ve.insert_or_update(id, LargeComponent::default());
        }
        b.iter(move || {
            let it = JoinIterator::new(bt.iter(), ve.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_vec_vec_2pow15(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut ta = VecTable::<EntityId, LargeComponent>::new();
        let mut tb = VecTable::<EntityId, LargeComponent>::new();
        for _ in 0..1 << 15 {
            let mut res = false;
            let mut id = EntityId::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 16));
                res = ta.insert_or_update(id, LargeComponent::default());
            }
            tb.insert_or_update(id, LargeComponent::default());
        }
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_bt_bt_2pow15(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut ta = BTreeTable::<EntityId, LargeComponent>::new();
        let mut tb = BTreeTable::<EntityId, LargeComponent>::new();
        for _ in 0..1 << 15 {
            let mut res = false;
            let mut id = EntityId::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 16));
                res = ta.insert_or_update(id, LargeComponent::default());
            }
            tb.insert_or_update(id, LargeComponent::default());
        }
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }
}
