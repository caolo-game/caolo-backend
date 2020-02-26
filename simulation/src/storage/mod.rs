mod macros;
pub mod views;

use crate::tables::{Component, TableId};
use views::{UnsafeView, View};

pub trait HasTable<Id: TableId, Row: Component<Id>> {
    fn view<'a>(&'a self) -> View<'a, Id, Row>;
    fn unsafe_view(&mut self) -> UnsafeView<Id, Row>;
}

pub trait Epic<Id: TableId> {
    fn delete(&mut self, key: &Id);
}
