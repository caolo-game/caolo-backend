//! The game state is represented by a relational model.
//! Tables are generic collections that store game data split by (shape) components.
//!
mod inmemory;
mod iterators;
pub use self::inmemory::*;
pub use self::iterators::*;
use crate::model::{self, Circle, EntityId, PositionComponent, UserData, UserId};

/// TableIds may be used as indices of tables
pub trait TableId:
    'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug
{
}
impl<T: 'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug>
    TableId for T
{
}

/// TableRows may be used as the row type of a table
pub trait TableRow: 'static + Clone + Send + std::fmt::Debug {}
impl<T: 'static + Clone + Send + std::fmt::Debug> TableRow for T {}

/// Components define both their shape (via their type) and the storage backend that shall be used to
/// store them.
pub trait Component<Id: TableId>: TableRow {
    type Table: Table<Row = Self> + Send + std::fmt::Debug;
}

pub trait Table {
    type Id: TableId;
    type Row: TableRow;

    fn get_by_id<'a>(&'a self, id: &Self::Id) -> Option<&'a Self::Row>;
    fn get_by_ids<'a>(&'a self, id: &[Self::Id]) -> Vec<(Self::Id, &'a Self::Row)>;

    /// insert or update
    /// return wether the item was inserted
    fn insert(&mut self, id: Self::Id, row: Self::Row) -> bool;

    fn delete(&mut self, id: &Self::Id) -> Option<Self::Row>;
}

pub trait UserDataTable {
    fn create_new(&mut self, row: UserData) -> UserId;
}

pub trait PositionTable {
    /// Vision is AABB with topleft - bottomleft points
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)>;
    fn count_entities_in_range(&self, vision: &Circle) -> usize;
}

pub trait LogTable {
    fn get_logs_by_time(&self, time: u64) -> Vec<(model::EntityTime, model::LogEntry)>;
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
