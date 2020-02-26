use crate::model::{
    components::{self, PositionComponent},
    geometry::Circle,
    indices, EntityId,
};
use serde::Serialize;

/// TableIds may be used as indices of tables
pub trait TableId:
    'static + Ord + PartialOrd + Eq + PartialEq + Copy + Default + Send + std::fmt::Debug + Serialize
{
}

/// Implement for Ids that are incremented in a serial fashion.
/// Intended to be used in tables that reserve memory for N items where N is the largest Id
/// inserted.
/// e.g. inserting ids 0, 3, 4, 10 will reserve memory for 11 ([0..10]) items.
pub trait SerialId: TableId {
    /// Return the next Id in the domain after `self`.
    fn next(&self) -> Self;
    fn as_usize(&self) -> usize;
}

impl<T> TableId for T where
    T: 'static
        + Ord
        + PartialOrd
        + Eq
        + PartialEq
        + Copy
        + Default
        + Send
        + std::fmt::Debug
        + Serialize
{
}

/// TableRows may be used as the row type of a table
pub trait TableRow: 'static + Clone + Send + std::fmt::Debug + Serialize {}
impl<T: 'static + Clone + Send + std::fmt::Debug + Serialize> TableRow for T {}

/// Components define both their shape (via their type) and the storage backend that shall be used to
/// store them.
pub trait Component<Id: TableId>: TableRow {
    type Table: Table<Row = Self> + Send + std::fmt::Debug + Default + Serialize;
}

pub trait Table {
    type Id: TableId;
    type Row: TableRow;

    fn delete(&mut self, id: &Self::Id) -> Option<Self::Row>;
}

pub trait PositionTable {
    /// Vision is AABB with topleft - bottomleft points
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)>;
    fn count_entities_in_range(&self, vision: &Circle) -> usize;
}

pub trait LogTable {
    fn get_logs_by_time(&self, time: u64) -> Vec<(indices::EntityTime, components::LogEntry)>;
}
