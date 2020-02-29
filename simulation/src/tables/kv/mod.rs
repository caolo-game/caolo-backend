//! Key-Value storages.
//!
mod btree;
mod morton;
mod vector;

pub use btree::*;
pub use morton::*;
pub use vector::*;

use super::*;

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

    fn random_vec_table(len: usize, domain: u32) -> VecTable<EntityId, LargeComponent> {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::with_capacity(domain as usize);
        for _ in 0..len {
            let mut res = false;
            while !res {
                let id = EntityId(rng.gen_range(0, domain));
                res = table.insert_or_update(id, LargeComponent::default());
            }
        }
        table
    }

    fn random_bt_table(len: usize, domain: u32) -> BTreeTable<EntityId, LargeComponent> {
        let mut rng = rand::thread_rng();
        let mut table = BTreeTable::new();
        for _ in 0..len {
            let mut res = false;
            while !res {
                let id = EntityId(rng.gen_range(0, domain));
                res = table.insert_or_update(id, LargeComponent::default());
            }
        }
        table
    }

    #[bench]
    fn join_vec_btree_2pow15_sparse(b: &mut Bencher) {
        let bt = random_bt_table(1 << 15, 1 << 16);
        let ve = random_vec_table(1 << 15, 1 << 16);
        b.iter(move || {
            let it = JoinIterator::new(ve.iter(), bt.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_btree_vec_2pow15_sparse(b: &mut Bencher) {
        let bt = random_bt_table(1 << 15, 1 << 16);
        let ve = random_vec_table(1 << 15, 1 << 16);
        b.iter(move || {
            let it = JoinIterator::new(bt.iter(), ve.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_vec_vec_2pow15_sparse(b: &mut Bencher) {
        let ta = random_vec_table(1 << 15, 1 << 16);
        let tb = random_vec_table(1 << 15, 1 << 16);
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_bt_bt_2pow15_sparse(b: &mut Bencher) {
        let ta = random_bt_table(1 << 15, 1 << 16);
        let tb = random_bt_table(1 << 15, 1 << 16);
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_vec_btree_2pow15_dense(b: &mut Bencher) {
        let bt = random_bt_table(1 << 15, 1 << 15);
        let ve = random_vec_table(1 << 15, 1 << 15);
        b.iter(move || {
            let it = JoinIterator::new(ve.iter(), bt.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_btree_vec_2pow15_dense(b: &mut Bencher) {
        let bt = random_bt_table(1 << 15, 1 << 15);
        let ve = random_vec_table(1 << 15, 1 << 15);
        b.iter(move || {
            let it = JoinIterator::new(bt.iter(), ve.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_vec_vec_2pow15_dense(b: &mut Bencher) {
        let ta = random_vec_table(1 << 15, 1 << 15);
        let tb = random_vec_table(1 << 15, 1 << 15);
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }

    #[bench]
    fn join_bt_bt_2pow15_dense(b: &mut Bencher) {
        let ta = random_bt_table(1 << 15, 1 << 15);
        let tb = random_bt_table(1 << 15, 1 << 15);
        b.iter(move || {
            let it = JoinIterator::new(tb.iter(), ta.iter());
            for joined in it {
                black_box(joined);
            }
        });
    }
}
