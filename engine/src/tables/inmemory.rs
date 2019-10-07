use super::*;
use crate::storage::TableId;
use rayon::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct InMemoryTable<Id, Row>
where
    Id: TableId,
    Row: Clone,
{
    data: BTreeMap<Id, Row>,
}

impl<Id, Row> InMemoryTable<Id, Row>
where
    Id: TableId,
    Row: Clone,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}

impl<Id, Row> TableBackend for InMemoryTable<Id, Row>
where
    Id: TableId,
    Row: Clone,
{
    type Id = Id;
    type Row = Row;

    fn get_by_id(&self, id: &Id) -> Option<Row> {
        self.data.get(id).cloned()
    }

    fn get_by_ids(&self, ids: &[Id]) -> Vec<(Id, Row)> {
        self.data
            .iter()
            .filter(move |(i, _)| ids.iter().any(|x| *i == x))
            .map(move |(i, v)| (*i, v.clone()))
            .collect()
    }

    fn insert(&mut self, id: Id, row: Row) {
        self.data.insert(id, row);
    }

    fn delete(&mut self, id: &Id) -> Option<Row> {
        self.data.remove(id)
    }

    fn iter<'a>(&'a self) -> Box<dyn TableIterator<Id, Row> + 'a> {
        Box::new(self.data.iter().map(|(id, row)| (*id, row.clone())))
    }
}

impl BotTable for InMemoryTable<EntityId, Bot> {
    fn get_bots_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Bot)> {
        self.data
            .par_iter()
            .filter(|(_, e)| e.owner_id.map(|id| id == *user_id).unwrap_or(false))
            .map(|(id, e)| (*id, e.clone()))
            .collect()
    }
}

impl StructureTable for InMemoryTable<EntityId, Structure> {
    fn get_structures_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Structure)> {
        self.data
            .par_iter()
            .filter(|(_, e)| e.owner_id.map(|id| id == *user_id).unwrap_or(false))
            .map(|(id, e)| (*id, e.clone()))
            .collect()
    }
}

impl UserDataTable for InMemoryTable<UserId, UserData> {
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
        self.insert(id, row);
        id
    }
}

impl PositionTable for InMemoryTable<EntityId, PositionComponent> {
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
