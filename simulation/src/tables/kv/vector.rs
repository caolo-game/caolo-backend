//! Table with `Vec` back-end. Optimised for dense storage.
//! The storage will allocate memory for N items where `N = the largest id inserted`.
//! Because of this one should use this if the domain of the ids is small and/or dense.
//! Note that the `delete` operation is extremely slow, one should prefer updates to deletions.
//!
use super::*;
use rayon::prelude::*;
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::mem;

#[derive(Default, Debug, Serialize)]
pub struct VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    /// Id, index pairs
    keys: Vec<Option<(Id, u32)>>,
    values: Vec<Row>,
}

impl<Id, Row> VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    pub fn new() -> Self {
        let size = mem::size_of::<Id>().max(mem::size_of::<Row>());
        // reserve at most 1024 * 2 bytes
        let size = 1024 / size;
        Self {
            keys: Vec::with_capacity(size),
            values: Vec::with_capacity(size),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let size = mem::size_of::<Id>().max(mem::size_of::<Row>());
        let size = 1024 / size;
        Self {
            keys: Vec::with_capacity(cap),
            values: Vec::with_capacity(cap.min(size)),
        }
    }

    pub fn insert_or_update(&mut self, id: Id, row: Row) -> bool {
        let i = id.as_usize();
        let len = self.keys.len();
        // Extend the vector if necessary
        if i >= len {
            self.keys.resize(i + 1, None);
        } else if let Some((_, ind)) = self.keys[i] {
            self.values[ind as usize] = row;
            return true;
        }
        self.keys[i] = Some((id, u32::try_from(self.values.len()).unwrap()));
        self.values.push(row);

        true
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        let ind = id.as_usize();
        self.keys
            .get(ind)
            .and_then(|key| key.map(|(_, ind)| &self.values[ind as usize]))
    }

    pub fn iter<'a>(&'a self) -> impl TableIterator<Id, &'a Row> + 'a {
        let values = self.values.as_ptr();
        self.keys.iter().filter_map(|k| *k).map(move |(id, ind)| {
            let val = unsafe { &*values.offset(ind as isize) };
            (id, val)
        })
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl TableIterator<Id, &'a mut Row> + 'a {
        let values = self.values.as_mut_ptr();
        self.keys.iter().filter_map(|k| *k).map(move |(id, ind)| {
            let val = unsafe { &mut *values.offset(ind as isize) };
            (id, val)
        })
    }

    pub fn contains_id(&self, id: &Id) -> bool {
        let i = id.as_usize();
        self.keys.get(i).and_then(|x| x.as_ref()).is_some()
    }
}

impl<Id, Row> Table for VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    type Id = Id;
    type Row = Row;

    fn delete(&mut self, id: &Id) -> Option<Row> {
        if !self.contains_id(id) {
            return None;
        }
        let ind = id.as_usize();
        let i = self.keys[ind].unwrap().1 as usize;
        let limes = self.values.len() - 1;
        if i == limes {
            // the value is the last one
            self.keys[ind] = None;
            return self.values.pop();
        }
        // find the id of the last value
        let last = self
            .keys
            .par_iter_mut()
            .filter_map(|x| x.as_mut())
            .find_any(|(_, ind)| *ind as usize == limes)
            .expect("id corresponding to the last value");

        self.values.swap(i, limes);
        last.1 = i as u32;

        self.keys[ind] = None;
        self.values.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EntityId;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use test::Bencher;

    #[bench]
    fn insert_at_random(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert_or_update(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
    }

    #[bench]
    fn insert_at_random_w_reserve(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::with_capacity(1 << 20);
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert_or_update(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
    }

    #[bench]
    fn insert_at_random_w_median(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        table.insert_or_update(EntityId((1 << 20) / 2), 512);
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert_or_update(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
    }

    #[bench]
    fn update_all_iter_2pow14_sparse(b: &mut Bencher) {
        /// The Id domain is 1.2 * LEN
        ///
        const LEN: usize = 1 << 14;
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, usize>::with_capacity(LEN);
        for i in 0..LEN {
            let mut id = Default::default();
            while table.contains_id(&id) {
                id = EntityId(rng.gen_range(
                    0,
                    u32::try_from(LEN * 6 / 5).expect("max len to fit into u32"),
                ));
            }
            table.insert_or_update(id, i);
        }
        b.iter(|| {
            table.iter_mut().for_each(|(_, val)| {
                *val += 8;
                test::black_box(val);
            });
        });
    }

    #[bench]
    fn update_all_iter_2pow14_dense(b: &mut Bencher) {
        /// The whole table is filled
        ///
        const LEN: usize = 1 << 14;
        let mut table = VecTable::<EntityId, usize>::with_capacity(LEN);
        for i in 0..LEN {
            let id = EntityId(i as u32);
            table.insert_or_update(id, i);
        }
        b.iter(|| {
            table.iter_mut().for_each(|(_, val)| {
                *val += 8;
                test::black_box(val);
            });
        });
    }

    #[bench]
    fn get_by_id_random(b: &mut Bencher) {
        const LEN: usize = 1 << 20;
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, usize>::with_capacity(LEN);
        let mut ids = Vec::with_capacity(LEN);
        for i in 0..LEN {
            let mut id = Default::default();
            while table.contains_id(&id) {
                id = EntityId(
                    rng.gen_range(0, u32::try_from(LEN * 2).expect("max len to fit into u32")),
                );
            }
            table.insert_or_update(id, i);
            ids.push((id, i));
        }
        b.iter(|| {
            let ind = rng.gen_range(0, LEN);
            let (id, x) = ids[ind];
            let res = table.get_by_id(&id);
            debug_assert_eq!(*res.expect("result to be found"), x);
            res
        });
    }

    #[bench]
    fn override_update_random(b: &mut Bencher) {
        const LEN: usize = 1 << 20;
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, usize>::with_capacity(LEN);
        let mut ids = Vec::with_capacity(LEN);
        for i in 0..LEN {
            let mut id = Default::default();
            while table.contains_id(&id) {
                id = EntityId(
                    rng.gen_range(0, u32::try_from(LEN * 2).expect("max len to fit into u32")),
                );
            }
            table.insert_or_update(id, i);
            ids.push((id, i));
        }
        b.iter(|| {
            let ind = rng.gen_range(0, LEN);
            let (id, x) = ids[ind];
            let res = table.insert_or_update(id, x * 2);
            debug_assert!(res);
            res
        });
    }

    #[bench]
    fn delete_by_id_random(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        let mut ids = Vec::with_capacity(1 << 15);
        for i in 0..1 << 15 {
            let mut res = false;
            let mut id = Default::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 25));
                res = table.insert_or_update(id, i);
            }
            ids.push(id);
        }
        ids.as_mut_slice().shuffle(&mut rng);
        b.iter(|| {
            let id = ids.pop().expect("out of ids");
            let res = table.delete(&id);
            debug_assert!(res.is_some());
            res
        });
    }
}
