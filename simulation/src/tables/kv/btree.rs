use super::*;
use crate::storage::TableId;
use rayon::prelude::*;
use serde_derive::Serialize;
use std::collections::BTreeMap;

#[derive(Default, Debug, Serialize)]
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

    pub fn iter_mut<'a>(&'a mut self) -> impl TableIterator<Id, &'a mut Row> + 'a {
        self.data.iter_mut().map(|(id, row)| (*id, row))
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        self.data.get(id)
    }

    pub fn get_by_ids<'a>(&'a self, ids: &[Id]) -> Vec<(Id, &'a Row)> {
        self.data
            .iter()
            .filter(move |(i, _)| ids.iter().any(|x| *i == x))
            .map(move |(i, v)| (*i, v))
            .collect()
    }

    pub fn insert_or_update(&mut self, id: Id, row: Row) -> bool {
        self.data.insert(id, row);
        true
    }
}

impl<Id, Row> Table for BTreeTable<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    type Id = Id;
    type Row = Row;

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
        self.insert_or_update(id, row);
        id
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
