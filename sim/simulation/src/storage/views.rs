//! Views are designed to be used as function parameters where functions depend on tables in a
//! Storage. They are intended to be used to display data dependencies in the function signatures.
//!
//! Using tuples of views:
//!
//! ```
//! use caolo_sim::prelude::*;
//!
//! fn update_minerals(
//!     (mut entity_positions, mut energy): (
//!         UnsafeView<EntityId, PositionComponent>,
//!         UnsafeView<EntityId, EnergyComponent>,
//!     ),
//!     (position_entities, resources): (
//!         View<WorldPosition, EntityComponent>,
//!         View<EntityId, ResourceComponent>,
//!     ),
//! ) {
//!     // do stuff
//! }
//!
//! let mut storage = World::new(None);
//! update_minerals(FromWorldMut::new(&mut storage), FromWorld::new(&storage));
//! ```
//!
mod unsafe_view;
mod unwrap;
mod unwrap_mut;
mod view;
mod world_logger;

pub use unsafe_view::*;
pub use unwrap::*;
pub use unwrap_mut::*;
pub use view::*;
pub use world_logger::*;

use super::{Component, DeleteById, TableId};
use crate::indices::EntityId;
use crate::prelude::World;
use std::ptr::NonNull;

pub trait FromWorld<'a> {
    fn new(w: &'a World) -> Self;
}

pub trait FromWorldMut {
    fn new(w: &mut World) -> Self;
}

#[derive(Clone, Copy)]
pub struct DeferredDeleteEntityView {
    world: NonNull<World>,
}

unsafe impl Send for DeferredDeleteEntityView {}
unsafe impl Sync for DeferredDeleteEntityView {}

impl DeferredDeleteEntityView
where
    crate::world::World: super::DeferredDeleteById<EntityId>,
{
    /// # Safety
    /// This function should only be called if the pointed to Storage is in memory and no other
    /// threads have access to it at this time!
    pub unsafe fn delete_entity(&mut self, id: EntityId) {
        use super::DeferredDeleteById;

        let world = self.world.as_mut();
        world.deferred_delete(id);
    }
}

impl FromWorldMut for DeferredDeleteEntityView {
    fn new(w: &mut World) -> Self {
        Self {
            world: unsafe { NonNull::new_unchecked(w) },
        }
    }
}

#[derive(Clone, Copy)]
pub struct DeleteEntityView {
    storage: NonNull<World>,
}

unsafe impl Send for DeleteEntityView {}
unsafe impl Sync for DeleteEntityView {}

impl DeleteEntityView
where
    crate::world::entity_store::Archetype: super::DeleteById<EntityId>,
{
    /// # Safety
    /// This function should only be called if the pointed to Storage is in memory and no other
    /// threads have access to it at this time!
    pub unsafe fn delete_entity(&mut self, id: EntityId) {
        let storage = &mut self.storage.as_mut().entities;
        storage.delete(id);
    }
}

impl FromWorldMut for DeleteEntityView {
    fn new(w: &mut World) -> Self {
        Self {
            storage: unsafe { NonNull::new_unchecked(w) },
        }
    }
}

#[derive(Clone, Copy)]
pub struct InsertEntityView {
    storage: NonNull<World>,
}

unsafe impl Send for InsertEntityView {}
unsafe impl Sync for InsertEntityView {}

impl FromWorldMut for InsertEntityView {
    fn new(w: &mut World) -> Self {
        Self {
            storage: unsafe { NonNull::new_unchecked(w) },
        }
    }
}

impl InsertEntityView {
    /// # Safety
    /// This function should only be called if the pointed to Storage is in memory and no other
    /// threads have access to it at this time!
    pub unsafe fn insert_entity(&mut self) -> EntityId {
        let storage = self.storage.as_mut();
        storage.insert_entity()
    }
}

macro_rules! implement_tuple {
    ($id: tt = $v: ident) => {
        impl<'a, $v: FromWorld<'a> >
            FromWorld <'a> for ( $v, )
            {
                #[allow(unused)]
                fn new(storage: &'a World) -> Self {
                    (
                        $v::new(storage) ,
                    )
                }
            }

        impl<$v:FromWorldMut >
            FromWorldMut  for ( $v, )
            {
                #[allow(unused)]
                fn new(storage: &mut World) -> Self {
                    (
                        $v::new(storage),
                    )
                }

            }
    };

    ($($id: tt = $vv: ident),*) => {
        impl<'a, $($vv:FromWorld<'a>),* >
            FromWorld <'a> for ( $($vv),* )
            {
                #[allow(unused)]
                fn new(storage: &'a World) -> Self {
                    (
                        $($vv::new(storage)),*
                    )
                }
            }

        impl<'a, $($vv:FromWorldMut),* >
            FromWorldMut  for ( $($vv),* )
            {
                #[allow(unused)]
                fn new(storage: &mut World) -> Self {
                    (
                        $($vv::new(storage)),*
                    )
                }
            }
    };
}

implement_tuple!();
implement_tuple!(0 = V1);
implement_tuple!(0 = V1, 1 = V2);
implement_tuple!(0 = V1, 1 = V2, 2 = V3);
implement_tuple!(0 = V1, 1 = V2, 2 = V3, 3 = V4);
implement_tuple!(0 = V1, 1 = V2, 2 = V3, 3 = V4, 4 = V5);
implement_tuple!(0 = V1, 1 = V2, 2 = V3, 3 = V4, 4 = V5, 5 = V6);
implement_tuple!(0 = V1, 1 = V2, 2 = V3, 3 = V4, 4 = V5, 5 = V6, 6 = V7);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9,
    9 = V10
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9,
    9 = V10,
    10 = V11
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9,
    9 = V10,
    10 = V11,
    11 = V12
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9,
    9 = V10,
    10 = V11,
    11 = V12,
    12 = V13
);
implement_tuple!(
    0 = V1,
    1 = V2,
    2 = V3,
    3 = V4,
    4 = V5,
    5 = V6,
    6 = V7,
    7 = V8,
    8 = V9,
    9 = V10,
    10 = V11,
    11 = V12,
    12 = V13,
    13 = V14
);
