use super::*;
use crate::storage::TableId;
use rayon::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    data: BTreeMap<Id, Row>,
}

impl<Id, Row> BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn iter<'a>(&'a self) -> impl TableIterator<Id, &'a Row> + 'a {
        self.data.iter().map(|(id, row)| (*id, row))
    }
}

impl<Id, Row> Table for BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    type Id = Id;
    type Row = Row;

    fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        self.data.get(id)
    }

    fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        self.data
            .iter()
            .filter(move |(i, _)| ids.iter().any(|x| *i == x))
            .map(move |(i, v)| (*i, v))
            .collect()
    }

    fn insert(&mut self, id: Id, row: Row) {
        self.data.insert(id, row);
    }

    fn delete(&mut self, id: &Id) -> Option<Row> {
        self.data.remove(id)
    }
}

impl UserDataTable for BTreeTable<UserId, UserData> {
    fn create_new(&mut self, row: UserData) -> UserId {
        use rand::RngCore;
        use uuid::{Builder, Variant, Version};

        let mut rng = rand::thread_rng();
        let mut random_bytes = [0; 16];
        rng.fill_bytes(&mut random_bytes);

        let id = Builder::from_slice(&random_bytes)
            .unwrap()
            .set_variant(Variant::RFC4122)
            .set_version(Version::Random)
            .build();
        let id = UserId(id);
        self.insert(id, row);
        id
    }
}

impl PositionTable for BTreeTable<EntityId, PositionComponent> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        self.data
            .par_iter()
            .filter(|(_, p)| p.0.hex_distance(vision.center) <= u64::from(vision.radius))
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        self.data
            .par_iter()
            .filter(|(_, p)| p.0.hex_distance(vision.center) <= u64::from(vision.radius))
            .count()
    }
}

impl LogTable for BTreeTable<model::EntityTime, model::LogEntry> {
    fn get_logs_by_time(&self, time: u64) -> Vec<(model::EntityTime, model::LogEntry)> {
        self.data
            .par_iter()
            .filter(|(t, _)| t.1 == time)
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caolo_api::point::Point;
    use rand::prelude::*;
    use test::Bencher;

    #[bench]
    fn bench_get_entities_in_range(b: &mut Bencher) {
        let mut rng = rand::thread_rng();

        let mut table = BTreeTable::new();

        for i in 0..(1 << 15) {
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            let p = PositionComponent(p);
            table.insert(EntityId(rng.gen()), p);
        }

        let radius = 512;

        b.iter(|| {
            let table = &table;
            let p = Point {
                x: rng.gen_range(-3900, 3900),
                y: rng.gen_range(-3900, 3900),
            };
            table.get_entities_in_range(&Circle { center: p, radius })
        });
    }
}
