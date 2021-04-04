use prelude::{Component, World};
use tables::{unique::UniqueTable, TableId};

pub mod components;
pub mod diagnostics;
pub mod executor;
pub mod geometry;
pub mod indices;
pub mod init;
pub mod map_generation;
pub mod noise;
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

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_sim_version.rs"));
}

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
#[cfg(not(feature = "tokio"))]
#[derive(Clone, Default)]
pub struct RuntimeGuard(std::marker::PhantomData<()>);

#[cfg(feature = "tokio")]
#[derive(Clone)]
pub struct RuntimeGuard(std::sync::Arc<tokio::runtime::Runtime>);

#[cfg(feature = "tokio")]
impl Default for RuntimeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeGuard {
    #[cfg(not(feature = "tokio"))]
    pub fn new() -> RuntimeGuard {
        RuntimeGuard(Default::default())
    }

    #[cfg(feature = "tokio")]
    pub fn new() -> RuntimeGuard {
        use std::sync::Arc;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(3.max(rayon::current_num_threads() / 4)) // leave more resources for our boi rayon.
            .enable_all()
            .build()
            .expect("Failed to init tokio runtime");
        RuntimeGuard(Arc::new(rt))
    }
}

#[cfg(feature = "tokio")]
impl RuntimeGuard {
    pub fn block_on<F>(&self, f: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.0.block_on(f)
    }

    pub fn runtime(&self) -> &std::sync::Arc<tokio::runtime::Runtime> {
        &self.0
    }
}
