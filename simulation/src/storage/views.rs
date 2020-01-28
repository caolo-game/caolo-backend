use super::{Component, EntityId, EntityTime, Point, ScriptId, Storage, TableId, UserId};
use std::ops::{Deref, DerefMut};

/// Fetch read-only tables from a Storage
///
/// ```
/// use caolo_sim::model::{EntityId, Bot, SpawnComponent};
/// use caolo_sim::storage::{views::View, Storage};
/// use caolo_sim::tables::BTreeTable;
///
/// let mut storage = Storage::new();
/// storage.add_entity_table::<Bot>(BTreeTable::new());
/// storage.add_entity_table::<SpawnComponent>(BTreeTable::new());
///
/// fn consumer(b: View<EntityId, Bot>, s: View<EntityId, SpawnComponent>) {
///   let bot_component = b.get_by_id(&EntityId::default());
///   let spawn_component = s.get_by_id(&EntityId::default());
/// }
///
/// let storage = &storage;
/// consumer(storage.into(), storage.into());
/// ```
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
/// Intended to be used by subsystems operating on multiple tables to better convey their data
/// dependencies in function signatures.
/// Do not store UnsafeViews for longer than the function scope, that's just asking for trouble.
///
/// ```
/// use caolo_sim::model::{EntityId, Bot,CarryComponent};
/// use caolo_sim::storage::{views::{View, UnsafeView}, Storage};
/// use caolo_sim::tables::BTreeTable;
///
/// let mut storage = Storage::new();
/// storage.add_entity_table::<Bot>(BTreeTable::new());
/// storage.add_entity_table::<CarryComponent>(BTreeTable::new());
///
/// // obtain a writable reference to the CarryComponent table and a read-only reference to the Bot
/// // table
/// fn consumer(mut carry: UnsafeView<EntityId, CarryComponent>, bot: View<EntityId, Bot>) {
///   let bot_component = bot.get_by_id(&EntityId::default());
///   carry.insert_or_update(EntityId::default(), Default::default());
/// }
///
/// consumer(UnsafeView::from(&mut storage),View::from(&storage));
/// ```
pub struct UnsafeView<Id: TableId, C: Component<Id>>(*mut C::Table);

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

impl<Id: TableId, C: Component<Id>> DerefMut for UnsafeView<Id, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
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

        impl<C: Component<$id>> HasNewMut for UnsafeView<$id, C> {
            fn new(storage: &mut Storage) -> Self {
                Self(storage.$field_mut::<C>() as *mut _)
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
