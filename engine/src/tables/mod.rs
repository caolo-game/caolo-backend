#[cfg(feature = "inmemory_storage")]
mod inmemory;
mod iterators;
#[cfg(feature = "inmemory_storage")]
pub use self::inmemory::*;
pub use self::iterators::*;
use crate::model::{Bot, Circle, EntityId, PositionComponent, Structure, UserId};
// TODO: remove caolo_api dependency from this module, replace with model/
use caolo_api::user::UserData;

pub trait TableId:
    'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug
{
}

impl<T: 'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug>
    TableId for T
{
}

pub trait TableRow: 'static + Clone + Send + std::fmt::Debug {}

impl<T: 'static + Clone + Send + std::fmt::Debug> TableRow for T {}

/// A generic proxy to the actual table implementation
pub struct Table<Id: TableId, Row: Clone> {
    backend: Backend<Id, Row>,
}

impl<Id, Row> Table<Id, Row>
where
    Id: TableId,
    Row: TableRow,
{
    #[cfg(feature = "inmemory_storage")]
    pub fn with_inmemory(table: InMemoryTable<Id, Row>) -> Self {
        Self {
            backend: Backend::InMemory(table),
        }
    }

    #[cfg(feature = "inmemory_storage")]
    pub fn default_inmemory() -> Self {
        Self {
            backend: Backend::InMemory(InMemoryTable::new()),
        }
    }

    pub fn get_by_id(&self, id: &Id) -> Option<Row> {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.get_by_id(id),
        }
    }

    pub fn get_by_ids(&self, id: &[Id]) -> Vec<(Id, Row)> {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.get_by_ids(id),
        }
    }

    /// insert or update
    pub fn insert(&mut self, id: Id, row: Row) {
        #[cfg(feature = "log_tables")]
        {
            use std::any::type_name;

            trace!(
                "Inserting [{:?}] : {:?} into {:?}",
                id,
                row,
                type_name::<Self>()
            );
        }

        use Backend::*;
        match &mut self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.insert(id, row),
        }
    }

    pub fn delete(&mut self, id: &Id) -> Option<Row> {
        #[cfg(feature = "log_tables")]
        {
            use std::any::type_name;

            trace!("Deleting [{:?}] from {:?}", id, type_name::<Self>());
        }

        use Backend::*;
        match &mut self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.delete(id),
        }
    }

    /// Contract: ids should be ordered
    // TODO: remove the need for heap allocation
    pub fn iter<'a>(&'a self) -> Box<dyn TableIterator<Id, Row> + 'a> {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.iter(),
        }
    }
}

enum Backend<Id: TableId, Row: Clone> {
    #[cfg(feature = "inmemory_storage")]
    InMemory(InMemoryTable<Id, Row>),
}

trait TableBackend {
    type Id: TableId;
    type Row: Clone;

    fn get_by_id(&self, id: &Self::Id) -> Option<Self::Row>;

    fn get_by_ids(&self, id: &[Self::Id]) -> Vec<(Self::Id, Self::Row)>;

    /// insert or update
    fn insert(&mut self, id: Self::Id, row: Self::Row);

    fn delete(&mut self, id: &Self::Id) -> Option<Self::Row>;

    /// Contract: ids should be ordered
    // TODO: remove the need for heap allocation
    fn iter<'a>(&'a self) -> Box<dyn TableIterator<Self::Id, Self::Row> + 'a>;
}

pub trait BotTable {
    fn get_bots_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Bot)>;
}

impl BotTable for Table<EntityId, Bot> {
    fn get_bots_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Bot)> {
        self.iter()
            .filter(|(_, e)| e.owner_id.map(|id| id == *user_id).unwrap_or(false))
            .map(|(id, e)| (id, e.clone()))
            .collect()
    }
}

pub trait UserDataTable {
    fn create_new(&mut self, row: UserData) -> UserId;
}

impl UserDataTable for Table<UserId, UserData> {
    fn create_new(&mut self, row: UserData) -> UserId {
        use Backend::*;
        match &mut self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.create_new(row),
        }
    }
}

pub trait StructureTable {
    fn get_structures_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Structure)>;
}

impl StructureTable for Table<EntityId, Structure> {
    fn get_structures_by_owner(&self, user_id: &UserId) -> Vec<(EntityId, Structure)> {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.get_structures_by_owner(user_id),
        }
    }
}

pub trait PositionTable {
    /// Vision is AABB with topleft - bottomleft points
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)>;
    fn count_entities_in_range(&self, vision: &Circle) -> usize;
}

impl PositionTable for Table<EntityId, PositionComponent> {
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)> {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.get_entities_in_range(vision),
        }
    }
    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        use Backend::*;
        match &self.backend {
            #[cfg(feature = "inmemory_storage")]
            InMemory(table) => table.count_entities_in_range(vision),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[derive(Debug, Clone, Copy)]
    struct Row1(i32);

    #[derive(Debug, Clone, Copy)]
    struct Row2(i32);

    #[test]
    fn join_iterator_simple() {
        type Id = u64;
        let mut t1 = InMemoryTable::<Id, Row1>::new();
        let mut t2 = InMemoryTable::<Id, Row2>::new();

        let expected = [
            (1, Row1(1), Row2(1)),
            (2, Row1(2), Row2(2)),
            (5, Row1(5), Row2(5)),
        ];

        for i in 0..8 {
            t1.insert(i, Row1(i.try_into().unwrap()));
            t2.insert(i, Row2(i.try_into().unwrap()));
        }

        t2.delete(&0);
        t1.delete(&3);
        t1.delete(&4);

        for (id, r1, r2) in expected.iter() {
            t1.insert(*id, *r1);
            t2.insert(*id, *r2);
        }

        let mut count = 0;
        for ((eid, e1, e2), (aid, (a1, a2))) in
            expected.iter().zip(JoinIterator::new(t1.iter(), t2.iter()))
        {
            count += 1;
            assert_eq!(*eid, aid);
            assert_eq!(e1.0, a1.0);
            assert_eq!(e2.0, a2.0);
        }
        assert_eq!(count, expected.len());
    }
}
