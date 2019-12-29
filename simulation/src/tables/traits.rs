use crate::model::{
    self, Circle, EntityComponent, EntityId, Point, PositionComponent, UserData, UserId,
};

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

    /// return wether the item was inserted successfully
    fn insert(&mut self, id: Self::Id, row: Self::Row) -> bool;

    fn delete(&mut self, id: &Self::Id) -> Option<Self::Row>;
}

pub trait UserDataTable {
    fn create_new(&mut self, row: UserData) -> UserId;
}

pub trait PositionTable: Table<Id = Point, Row = EntityComponent> {
    /// Vision is AABB with topleft - bottomleft points
    fn get_entities_in_range(&self, vision: &Circle) -> Vec<(EntityId, PositionComponent)>;
    fn count_entities_in_range(&self, vision: &Circle) -> usize;
}

pub trait LogTable {
    fn get_logs_by_time(&self, time: u64) -> Vec<(model::EntityTime, model::LogEntry)>;
}
