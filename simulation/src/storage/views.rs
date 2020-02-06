//! Views are designed to be used as function parameters where functions depend on tables in a
//! Storage. They are intended to be used to display data dependencies in the function signatures.
//!
//! Using tuples of views:
//!
//! ```
//! use caolo_sim::model::{EntityId, Bot, SpawnComponent,Point, self};
//! use caolo_sim::storage::{views::{View, UnsafeView}, Storage};
//! use caolo_sim::tables::{VecTable,BTreeTable, MortonTable};
//!
//! fn update_minerals(
//!     (mut entity_positions, mut energy): (
//!         UnsafeView<EntityId, model::PositionComponent>,
//!         UnsafeView<EntityId, model::EnergyComponent>,
//!     ),
//!     (position_entities, resources): (
//!         View<Point, model::EntityComponent>,
//!         View<EntityId, model::ResourceComponent>,
//!     ),
//! ) {
//!     // do stuff
//! }
//!
//! let mut storage = Storage::new();
//! storage.add_entity_table::<model::PositionComponent>(VecTable::new());
//! storage.add_entity_table::<model::EnergyComponent>(BTreeTable::new());
//! storage.add_point_table::<model::EntityComponent>(MortonTable::new());
//! storage.add_entity_table::<model::ResourceComponent>(BTreeTable::new());
//! update_minerals(From::from(&mut storage), From::from(&storage));
//! ```
//!
use super::{Component, EntityId, EntityTime, Point, ScriptId, Storage, TableId, UserId};
use std::ops::Deref;

/// Fetch read-only tables from a Storage
///
#[derive(Clone, Copy)]
pub struct View<'a, Id: TableId, C: Component<Id>>(&'a C::Table);

impl<'a, Id: TableId, C: Component<Id>> Deref for View<'a, Id, C> {
    type Target = C::Table;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Fetch read-write table reference from a Storage.
/// This is a pretty unsafe way to obtain mutable references. Use with caution.
/// Do not store UnsafeViews for longer than the function scope, that's just asking for trouble.
/// Using UnsafeView after the Storage is destroyed is UB!
///
pub struct UnsafeView<Id: TableId, C: Component<Id>>(*mut C::Table);

impl<Id: TableId, C: Component<Id>> UnsafeView<Id, C> {
    pub unsafe fn as_mut(&mut self) -> &mut C::Table {
        &mut *self.0
    }
}

impl<Id: TableId, C: Component<Id>> Clone for UnsafeView<Id, C> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<Id: TableId, C: Component<Id>> Copy for UnsafeView<Id, C> {}

impl<Id: TableId, C: Component<Id>> Deref for UnsafeView<Id, C> {
    type Target = C::Table;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

pub struct DeleteEntityView {
    storage: *mut Storage,
}

impl DeleteEntityView {
    pub unsafe fn delete_entity(&mut self, entity_id: EntityId) {
        (*self.storage).delete_entity(entity_id)
    }
}

impl HasNewMut for DeleteEntityView {
    fn new(storage: &mut Storage) -> Self {
        Self {
            storage: storage as *mut _,
        }
    }
}

pub trait HasNew<'a> {
    fn new(s: &'a Storage) -> Self;
}

pub trait HasNewMut {
    fn new(s: &mut Storage) -> Self;
}

/// Implement the Ctor and conversion methods for a given TableId
macro_rules! implement_id {
    ($field: ident, $field_mut: ident, $id: ty) => {
        impl<'a, C: Component<$id>> HasNew<'a> for View<'a, $id, C> {
            fn new(storage: &'a Storage) -> Self {
                Self(storage.$field::<C>())
            }
        }

        impl<C: Component<$id>> HasNewMut for UnsafeView<$id, C> {
            fn new(storage: &mut Storage) -> Self {
                Self(storage.$field_mut::<C>() as *mut _)
            }
        }

        impl<'a, C: Component<$id>> From<&'a Storage> for View<'a, $id, C> {
            fn from(s: &'a Storage) -> Self {
                Self::new(s)
            }
        }

        impl<'a, C: Component<$id>> From<&'a mut Storage> for View<'a, $id, C> {
            fn from(s: &'a mut Storage) -> Self {
                Self::new(s)
            }
        }

        impl<C: Component<$id>> From<&mut Storage> for UnsafeView<$id, C> {
            fn from(s: &mut Storage) -> Self {
                Self::new(s)
            }
        }
    };
}

implement_id!(entity_table, entity_table_mut, EntityId);
implement_id!(point_table, point_table_mut, Point);
implement_id!(user_table, user_table_mut, UserId);
implement_id!(scripts_table, scripts_table_mut, ScriptId);
implement_id!(log_table, log_table_mut, EntityTime);

macro_rules! implement_tuple {
    ($v: ident) => {
            impl<'a, $v:HasNew<'a> >
            From <&'a Storage> for ( $v, )
            {
                #[allow(unused)]
                fn from(storage: &'a Storage) -> Self {
                    (
                        $v::new(storage) ,
                    )
                }
            }

            impl<$v:HasNewMut >
            From <&mut Storage> for ( $v, )
            {
                #[allow(unused)]
                fn from(storage: &mut Storage) -> Self {
                    (
                        $v ::new(storage),
                    )
                }
            }

            impl<'a, $v:HasNew<'a> >
            HasNew<'a> for ( $v, )
            {
                fn new(storage: &'a Storage) -> Self {
                    Self::from(storage)
                }
            }

            impl<$v:HasNewMut>
            HasNewMut for ( $v, )
            {
                fn new(storage: &mut Storage) -> Self {
                    Self::from(storage)
                }
            }
    };

    ($($vv: ident),*) => {
            impl<'a, $($vv:HasNew<'a>),* >
            From <&'a Storage> for ( $($vv),* )
            {
                #[allow(unused)]
                fn from(storage: &'a Storage) -> Self {
                    (
                        $($vv ::new(storage)),*
                    )
                }
            }

            impl<$($vv:HasNewMut),* >
            From <&mut Storage> for ( $($vv),* )
            {
                #[allow(unused)]
                fn from(storage: &mut Storage) -> Self {
                    (
                        $($vv ::new(storage)),*
                    )
                }
            }

            impl<'a, $($vv:HasNew<'a>),* >
            HasNew<'a> for ( $($vv),* )
            {
                fn new(storage: &'a Storage) -> Self {
                    Self::from(storage)
                }
            }

            impl<$($vv:HasNewMut),* >
            HasNewMut for ( $($vv),* )
            {
                fn new(storage: &mut Storage) -> Self {
                    Self::from(storage)
                }
            }
    };
}

implement_tuple!();
implement_tuple!(V1);
implement_tuple!(V1, V2);
implement_tuple!(V1, V2, V3);
implement_tuple!(V1, V2, V3, V4);
implement_tuple!(V1, V2, V3, V4, V5);
implement_tuple!(V1, V2, V3, V4, V5, V6);
implement_tuple!(V1, V2, V3, V4, V5, V6, V7);
implement_tuple!(V1, V2, V3, V4, V5, V6, V7, V8);
implement_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9);
implement_tuple!(V1, V2, V3, V4, V5, V6, V7, V8, V9, V10);
