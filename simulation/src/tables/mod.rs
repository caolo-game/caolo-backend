mod inmemory;
mod iterators;
pub use self::inmemory::*;
pub use self::iterators::*;
use crate::model::{self, Circle, EntityId, PositionComponent, UserId};
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
    pub fn with_btree(table: BTreeTable<Id, Row>) -> Self {
        Self {
            backend: Backend::BTree(table),
        }
    }

    pub fn default_btree() -> Self {
        Self {
            backend: Backend::BTree(BTreeTable::new()),
        }
    }

    pub fn get_by_id<'a>(&'a self, id: &Id) -> Option<&'a Row> {
        use Backend::*;
        match &self.backend {
            BTree(table) => table.get_by_id(id),
        }
    }

    pub fn get_by_ids<'a>(&'a self, id: &[Id]) -> Vec<(Id, &'a Row)> {
        use Backend::*;
        match &self.backend {
            BTree(table) => table.get_by_ids(id),
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
            BTree(table) => table.insert(id, row),
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
            BTree(table) => table.delete(id),
        }
    }

    /// Contract: ids should be ordered
    // TODO: remove the need for heap allocation
    pub fn iter<'a>(&'a self) -> Box<dyn TableIterator<Id, Row> + 'a> {
        use Backend::*;
        match &self.backend {
            BTree(table) => table.iter(),
        }
    }
}

enum Backend<Id: TableId, Row: Clone> {
    BTree(BTreeTable<Id, Row>),
}

pub trait TableBackend {
    type Id: TableId;
    type Row: Clone;

    fn get_by_id<'a>(&'a self, id: &Self::Id) -> Option<&'a Self::Row>;
    fn get_by_ids<'a>(&'a self, id: &[Self::Id]) -> Vec<(Self::Id, &'a Self::Row)>;

    /// insert or update
    fn insert(&mut self, id: Self::Id, row: Self::Row);

    fn delete(&mut self, id: &Self::Id) -> Option<Self::Row>;

    /// Contract: ids should be ordered
    // TODO: remove the need for heap allocation
    fn iter<'a>(&'a self) -> Box<dyn TableIterator<Self::Id, Self::Row> + 'a>;
}

pub trait UserDataTable {
    fn create_new(&mut self, row: UserData) -> UserId;
}

impl UserDataTable for Table<UserId, UserData> {
    fn create_new(&mut self, row: UserData) -> UserId {
        use Backend::*;
        match &mut self.backend {
            BTree(table) => table.create_new(row),
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
            BTree(table) => table.get_entities_in_range(vision),
        }
    }
    fn count_entities_in_range(&self, vision: &Circle) -> usize {
        use Backend::*;
        match &self.backend {
            BTree(table) => table.count_entities_in_range(vision),
        }
    }
}

pub trait LogTable {
    fn get_logs_by_time(&self, time: u64) -> Vec<((EntityId, u64), model::LogEntry)>;
}

impl LogTable for Table<(EntityId, u64), model::LogEntry> {
    fn get_logs_by_time(&self, time: u64) -> Vec<((EntityId, u64), model::LogEntry)> {
        use Backend::*;
        match &self.backend {
            BTree(table) => table.get_logs_by_time(time),
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
        let mut t1 = BTreeTable::<Id, Row1>::new();
        let mut t2 = BTreeTable::<Id, Row2>::new();

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
