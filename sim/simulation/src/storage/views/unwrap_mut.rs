use super::super::HasTable;
use super::{Component, FromWorldMut, UnsafeView, World};
use crate::tables::unique::UniqueTable;
use crate::tables::TableId;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct UnwrapViewMut<Id: TableId, C: Component<Id>>(NonNull<UniqueTable<Id, C>>);

impl<'a, Id: TableId, C: Component<Id>> Clone for UnwrapViewMut<Id, C> {
    fn clone(&self) -> Self {
        UnwrapViewMut(self.0)
    }
}

impl<'a, Id: TableId, C: Component<Id>> Copy for UnwrapViewMut<Id, C> {}

unsafe impl<'a, Id: TableId, C: Component<Id>> Send for UnwrapViewMut<Id, C> {}
unsafe impl<'a, Id: TableId, C: Component<Id>> Sync for UnwrapViewMut<Id, C> {}

impl<'a, Id: TableId, C: Component<Id>> UnwrapViewMut<Id, C> {
    pub fn from_table(t: &mut UniqueTable<Id, C>) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(t) };
        Self(ptr)
    }
}

impl<'a, Id: TableId, C: Component<Id>> Deref for UnwrapViewMut<Id, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
            .value
            .as_ref()
            .expect("UnwrapViewMut dereferenced with an empty table")
    }
}

impl<'a, Id: TableId, C: Component<Id>> DerefMut for UnwrapViewMut<Id, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
            .value
            .as_mut()
            .expect("UnwrapViewMut dereferenced with an empty table")
    }
}

impl<'a, Id: TableId, C: Default + Component<Id, Table = UniqueTable<Id, C>>> FromWorldMut
    for UnwrapViewMut<Id, C>
where
    crate::world::World: HasTable<Id, C>,
{
    fn new(w: &mut World) -> Self {
        let table = UnsafeView::new(w).as_ptr();
        UnwrapViewMut(NonNull::new(table).unwrap())
    }
}
