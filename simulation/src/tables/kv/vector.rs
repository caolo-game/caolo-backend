//! Table with `Vec` back-end. Optimised for dense storage.
//! The storage will allocate memory for N items where `N = the largest id inserted`.
//! Because of this one should use this if the domain of the ids is small or dense.
//!
use super::*;
use serde_derive::{Serialize, Deserialize};
use std::mem;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    data: Vec<Option<(Id, Row)>>,
    /// the `as_usize` index of the first item in the vector
    offset: usize,
}

impl<Id, Row> VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    pub fn new() -> Self {
        let size = mem::size_of::<(Id, Row)>();
        let size = 1024 / size;
        Self {
            offset: 0,
            data: Vec::with_capacity(size),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let size = mem::size_of::<(Id, Row)>();
        let size = 1024 / size;
        Self {
            offset: 0,
            data: Vec::with_capacity(size.min(cap)),
        }
    }

    pub fn insert_or_update(&mut self, id: Id, row: Row) -> bool {
        // Extend the vector if necessary
        let i = id.as_usize();
        let len = self.data.len();
        if i < self.offset {
            self.data.resize(self.offset - i + len, None);
            self.data.rotate_right(self.offset - i);
            self.offset = i;
        }
        let i = i - self.offset;
        if i >= len {
            self.data.resize(i + 1, None);
        }
        if let Some((_, r)) = self.data[i].as_mut() {
            *r = row;
        } else {
            self.data[i] = Some((id, row));
        }
        true
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        let ind = id.as_usize();
        if ind < self.offset {
            return None;
        }
        let ind = ind - self.offset;
        self.data
            .get(ind)
            .and_then(|x| x.as_ref().map(|(_, row)| row))
    }

    pub fn iter<'a>(&'a self) -> impl TableIterator<Id, &'a Row> + 'a {
        self.data
            .iter()
            .filter_map(|k| k.as_ref())
            .map(move |(id, row)| (*id, row))
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl TableIterator<Id, &'a mut Row> + 'a {
        self.data
            .iter_mut()
            .filter_map(|k| k.as_mut())
            .map(move |(id, row)| (*id, row))
    }

    pub fn contains_id(&self, id: &Id) -> bool {
        let i = id.as_usize();
        if i < self.offset {
            return false;
        }
        let i = i - self.offset;
        self.data.get(i).and_then(|x| x.as_ref()).is_some()
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
        let ind = id.as_usize() - self.offset;
        self.data.push(None);
        let res = self.data.swap_remove(ind);
        self.data.pop();
        res.map(|(_, row)| row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EntityId;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use std::convert::TryFrom;
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
    fn update_all_iter_2pow14_sparse(b: &mut Bencher) {
        // The Id domain is 1.2 * LEN

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
        // The whole table is filled

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
        let mut i = 0;
        let mask = (1 << 15) - 1;
        b.iter(|| {
            i = (i + 1) & mask;
            let id = ids[i];
            let res = table.delete(&id);
            debug_assert!(res.is_some());
            table.insert_or_update(id, 123);
            res
        });
    }
}
