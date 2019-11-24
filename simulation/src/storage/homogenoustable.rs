//! This modules will allow us to store arbitrary tables that share the same key
//!
use crate::tables::{Table, TableId, TableRow};
use std::any::{type_name, TypeId};
use std::fmt::{Debug, Formatter};

pub struct HomogenousTable<Id: TableId> {
    rowtype: TypeId,
    concrete_table: Box<dyn DynTable<Id> + 'static>,
}

impl<Id: TableId> HomogenousTable<Id> {
    pub fn downcast_ref<Row: TableRow>(&self) -> Option<&Table<Id, Row>> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference =
                unsafe { &*(self.concrete_table.as_ref() as *const _ as *const Table<Id, Row>) };
            Some(reference)
        } else {
            None
        }
    }

    pub fn downcast_mut<Row: TableRow>(&mut self) -> Option<&mut Table<Id, Row>> {
        if TypeId::of::<Row>() == self.rowtype {
            #[allow(clippy::cast_ptr_alignment)]
            // Yes, this is incredibly unsafe
            let reference =
                unsafe { &mut *(self.concrete_table.as_mut() as *mut _ as *mut Table<Id, Row>) };
            Some(reference)
        } else {
            None
        }
    }

    pub fn delete_entity(&mut self, id: &Id) {
        self.concrete_table.delete_entity(id);
    }

    pub fn new<Row: TableRow>(table: Table<Id, Row>) -> Self {
        let rowtype = TypeId::of::<Row>();
        let concrete_table = Box::new(table);
        Self {
            concrete_table,
            rowtype,
        }
    }
}

trait DynTable<Id: TableId> {
    fn delete_entity(&mut self, id: &Id);
}

impl<Id: TableId, Row: TableRow> DynTable<Id> for Table<Id, Row> {
    fn delete_entity(&mut self, id: &Id) {
        self.delete(id);
    }
}

impl<Id: 'static + TableId> Debug for HomogenousTable<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HomogenousTable id: {}, row: {:?}",
            type_name::<Id>(),
            self.rowtype
        )
    }
}
