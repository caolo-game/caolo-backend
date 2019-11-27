//! This modules will allow us to store arbitrary tables that share the same key
//!
use crate::tables::{self, Component, TableId, TableRow};
use std::any::{type_name, TypeId};
use std::fmt::{Debug, Formatter};

pub struct HomogenousTable<Id: TableId> {
    rowtype: TypeId,
    concrete_table: Box<dyn DynTable<Id> + 'static>,
}

impl<Id: TableId> HomogenousTable<Id> {
    pub fn downcast_ref<Row: Component<Id>>(&self) -> Option<&Row::Table> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference =
                unsafe { &*(self.concrete_table.as_ref() as *const _ as *const Row::Table) };
            Some(reference)
        } else {
            None
        }
    }

    pub fn downcast_mut<Row: Component<Id>>(&mut self) -> Option<&mut Row::Table> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference =
                unsafe { &mut *(self.concrete_table.as_mut() as *mut _ as *mut Row::Table) };
            Some(reference)
        } else {
            None
        }
    }

    pub fn delete_entity(&mut self, id: &Id) {
        self.concrete_table.delete_entity(id);
    }

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

pub trait DynTable<Id: TableId> {
    fn delete_entity(&mut self, id: &Id);
}

impl<Id: TableId, Row: TableRow, T: tables::Table<Id = Id, Row = Row>> DynTable<Id> for T {
    fn delete_entity(&mut self, id: &Id) {
        self.delete(id);
    }
}

impl<Id: 'static + TableId> Debug for HomogenousTable<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HomogenousTable index: {}, type: {:?}",
            type_name::<Id>(),
            self.rowtype
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Bot, EntityId};
    use crate::tables::BTreeTable;

    #[test]
    fn can_downcast() {
        let table = BTreeTable::new();
        let homo = HomogenousTable::<EntityId>::new::<Bot>(table);
        homo.downcast_ref::<Bot>().unwrap();
    }

    #[test]
    fn can_downcast_mut() {
        let table = BTreeTable::new();
        let mut homo = HomogenousTable::<EntityId>::new::<Bot>(table);
        homo.downcast_mut::<Bot>().unwrap();
    }
}
