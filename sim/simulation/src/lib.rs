use prelude::World;

pub mod components;
pub mod diagnostics;
pub mod entity_archetypes;
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

#[derive(Clone)]
pub struct RuntimeGuard(std::sync::Arc<tokio::runtime::Runtime>);

impl Default for RuntimeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeGuard {
    pub fn new() -> RuntimeGuard {
        use std::sync::Arc;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to init tokio runtime");
        RuntimeGuard(Arc::new(rt))
    }
}

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
