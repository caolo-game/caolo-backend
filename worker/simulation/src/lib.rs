use prelude::{Component, World};
use tables::{unique::UniqueTable, TableId};

pub mod components;
pub mod diagnostics;
pub mod executor;
pub mod geometry;
pub mod indices;
pub mod init;
pub mod map_generation;
pub mod pathfinding;
pub mod prelude;
pub mod scripting_api;
pub mod storage;
pub mod tables;
pub mod terrain;

mod intents;
mod systems;
mod utils;
pub mod world;

#[derive(Clone, Debug, Default, Copy, serde::Serialize, serde::Deserialize)]
pub struct Time(pub u64);

impl<'a> storage::views::FromWorld<'a> for Time {
    fn new(w: &'a World) -> Self {
        Time(w.time())
    }
}

impl<Id: TableId> Component<Id> for Time {
    type Table = UniqueTable<Id, Time>;
}

// use phantomdata member to disallow crate users the creation of runtimes directly
#[derive(Clone, Default)]
pub struct RuntimeGuard(std::marker::PhantomData<()>);

impl RuntimeGuard {
    pub fn new() -> RuntimeGuard {
        RuntimeGuard(Default::default())
    }
}

#[cfg(feature = "async-std")]
impl RuntimeGuard {
    pub fn block_on<F>(&self, f: F) -> F::Output
    where
        F: std::future::Future,
    {
        async_std::task::block_on(f)
    }
}
