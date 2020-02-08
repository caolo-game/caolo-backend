//! This modules will allow us to store arbitrary tables that share the same key
//!
use crate::tables::{self, Component, TableId, TableRow};
use std::any::{type_name, TypeId};
use std::fmt::{Debug, Formatter};

/// Proxy into a specific table.
/// HomogenousTables are only generic for the index parameter and can hide any table with the same
/// index.
pub struct HomogenousTable<Id: TableId> {
    rowtype: TypeId,
    concrete_table: Box<dyn DynTable<Id> + 'static>,
}

impl<Id: TableId> HomogenousTable<Id> {
    /// Downcast self as an immutable reference to the underlying table.
    pub fn downcast_ref<Row: Component<Id>>(&self) -> Option<&Row::Table> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference = unsafe {
                &*(self.concrete_table.as_ref() as *const dyn DynTable<Id> as *const Row::Table)
            };
            Some(reference)
        } else {
            None
        }
    }

    /// Downcast self as a mutable reference to the underlying table.
    pub fn downcast_mut<Row: Component<Id>>(&mut self) -> Option<&mut Row::Table> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference = unsafe {
                &mut *(self.concrete_table.as_mut() as *mut dyn DynTable<Id> as *mut Row::Table)
            };
            Some(reference)
        } else {
            None
        }
    }

    /// Delete a row that matches the index if any.
    pub fn delete_entity(&mut self, id: &Id) {
        self.concrete_table.delete_entity(id);
    }

    /// Create a new HomogenousTable given a back-end.
    pub fn new<Row: Component<Id>>(table: <Row as Component<Id>>::Table) -> Self
    where
        <Row as Component<Id>>::Table: DynTable<Id>,
    {
        let rowtype = TypeId::of::<Row>();
        let concrete_table = Box::new(table);
        Self {
            concrete_table,
            rowtype,
        }
    }
}

/// Generic methods all tables must provide when used in a homogenous environment.
/// This is automatically implemented for all types implementing `tables::Table`.
pub trait DynTable<Id: TableId> {
    fn delete_entity(&mut self, id: &Id);
}

impl<Id, Row, T> DynTable<Id> for T
where
    Id: TableId,
    Row: TableRow,
    T: tables::Table<Id = Id, Row = Row>,
{
    fn delete_entity(&mut self, id: &Id) {
        self.delete(id);
    }
}

impl<Id: 'static + TableId> Debug for HomogenousTable<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HomogenousTable index: {}, row type: {:?}",
            type_name::<Id>(),
            self.rowtype
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{components::Bot, EntityId};
    use crate::tables::VecTable;

    #[test]
    fn can_downcast() {
        let table = VecTable::new();
        let homo = HomogenousTable::<EntityId>::new::<Bot>(table);
        homo.downcast_ref::<Bot>().unwrap();
    }

    #[test]
    fn can_downcast_mut() {
        let table = VecTable::new();
        let mut homo = HomogenousTable::<EntityId>::new::<Bot>(table);
        homo.downcast_mut::<Bot>().unwrap();
    }
}
