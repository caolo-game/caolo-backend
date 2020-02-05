use super::*;
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::mem;

#[derive(Default, Debug, Serialize)]
pub struct VecTable<Id, Row>
where
    Id: SerialId,
    Row: TableRow,
{
    /// Id as usize at the 0th position
    min: usize,
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
            min: size,
            keys: Vec::with_capacity(size),
            values: Vec::with_capacity(size),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        let size = mem::size_of::<Id>().max(mem::size_of::<Row>());
        let size = 1024 / size;
        Self {
            min: 0,
            keys: Vec::with_capacity(cap),
            values: Vec::with_capacity(cap.min(size)),
        }
    }

    /// Returns true on successful insert and false on failure
    pub fn insert(&mut self, id: Id, row: Row) -> bool {
        let i = id.as_usize();
        let len = self.keys.len();
        // Padd the vector if necessary
        if i < self.min {
            let diff = self.min - i;
            self.keys.resize(len + diff, None);
            self.keys.rotate_left(diff);
            self.min = i;
        } else if i >= len {
            let diff = 1 + i - len;
            self.keys.resize(len + diff, None);
        } else if self.keys[i].is_some() {
            return false;
        }

        let ind = i - self.min;
        self.keys[ind] = Some((id, u32::try_from(self.values.len()).unwrap()));
        self.values.push(row);

        true
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        let ind = id.as_usize();
        if ind < self.min {
            return None;
        }
        let ind = ind - self.min;
        self.keys
            .get(ind)
            .and_then(|key| key.map(|(_, ind)| &self.values[ind as usize]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EntityId;
    use rand::Rng;
    use test::{black_box, Bencher};

    #[bench]
    fn insert_at_random(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
        black_box(table);
    }

    #[bench]
    fn insert_at_random_w_reserve(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::with_capacity(1 << 20);
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
        black_box(table);
    }

    #[bench]
    fn insert_at_random_w_median(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        table.insert(EntityId((1 << 20) / 2), 512);
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
        black_box(table);
    }

    #[bench]
    fn get_by_id_random(b: &mut Bencher) {
        let mut rng = rand::thread_rng();
        let mut table = VecTable::<EntityId, i32>::new();
        for i in 0..1 << 15 {
            let mut res = false;
            while !res {
                let id = rng.gen_range(0, 1 << 25);
                let id = EntityId(id);
                res = table.insert(id, i);
            }
        }
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 25);
            let id = EntityId(id);
            let res = table.get_by_id(&id);
            res
        });
        black_box(table);
    }
}
