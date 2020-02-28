#![feature(test)]
extern crate test;

#[macro_use]
extern crate log;

pub mod api;
pub mod model;
pub mod prelude;
pub mod storage;
pub mod tables;

mod intents;
mod systems;
mod utils;

use chrono::{DateTime, Duration, Utc};
use serde_derive::Serialize;
use storage::views::{UnsafeView, View};
use systems::execute_world_update;
use systems::intent_system::execute_intents;
use systems::script_execution::execute_scripts;
use tables::{Component, TableId};

pub fn forward(storage: &mut World) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing scripts");
    let final_intents = execute_scripts(storage);
    info!("Executing scripts - done");

    storage.signal_done(&final_intents);

    info!("Executing intents");
    execute_intents(final_intents, storage);
    info!("Executing intents - done");
    info!("Executing systems update");
    execute_world_update(storage);
    info!("Executing systems update - done");

    crate::utils::profiler::save_global();
    info!("-----------Tick {} done-----------", storage.time());
    Ok(())
}

mod data_store {
    use super::storage;
    use crate::model::components::*;
    use crate::model::geometry::Point;
    use crate::model::*;

    storage!(
        module store_impl

        key EntityId, table Bot = entitybot,
        key EntityId, table PositionComponent = entitypos,
        key EntityId, table SpawnBotComponent = entityspawnbot,
        key EntityId, table CarryComponent = entitycarry,
        key EntityId, table Structure = entitystructure,
        key EntityId, table HpComponent = entityhp,
        key EntityId, table EnergyRegenComponent = entityenergyregen,
        key EntityId, table EnergyComponent = entityenergy,
        key EntityId, table ResourceComponent = entityresource,
        key EntityId, table DecayComponent = entitydecay,
        key EntityId, table EntityScript = entityscript,
        key EntityId, table SpawnComponent = entityspawn,
        key EntityId, table OwnedEntity = entityowner,

        key EntityTime, table LogEntry = timelog,

        key UserId, table UserComponent = useruser,

        key Point, table TerrainComponent = pointterrain,
        key Point, table EntityComponent = pointentity,

        key ScriptId, table ScriptComponent = scriptscript
    );

    pub use self::store_impl::*;
}

#[derive(Debug, Serialize)]
pub struct World {
    store: data_store::Storage,

    time: u64,
    next_entity: crate::model::EntityId,
    last_tick: DateTime<Utc>,
    #[serde(skip)]
    dt: Duration,
}

impl<Id: TableId, C: Component<Id>> storage::HasTable<Id, C> for World
where
    data_store::Storage: storage::HasTable<Id, C>,
{
    fn view<'a>(&'a self) -> View<'a, Id, C> {
        self.store.view()
    }

    fn unsafe_view(&mut self) -> UnsafeView<Id, C> {
        self.store.unsafe_view()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

impl World {
    pub fn new() -> Self {
        let store = data_store::Storage::default();
        Self {
            time: 0,
            store,
            last_tick: Utc::now(),
            next_entity: crate::model::EntityId::default(),
            dt: Duration::zero(),
        }
    }

    pub fn view<'a, Id: TableId, C: Component<Id>>(&'a self) -> View<'a, Id, C>
    where
        data_store::Storage: storage::HasTable<Id, C>,
    {
        (&self.store as &dyn storage::HasTable<Id, C>).view()
    }

    pub fn unsafe_view<Id: TableId, C: Component<Id>>(&mut self) -> UnsafeView<Id, C>
    where
        data_store::Storage: storage::HasTable<Id, C>,
    {
        (&mut self.store as &mut dyn storage::HasTable<Id, C>).unsafe_view()
    }

    pub fn delete<Id: TableId>(&mut self, id: &Id)
    where
        data_store::Storage: storage::Epic<Id>,
    {
        let storage = &mut self.store as &mut dyn storage::Epic<Id>;
        storage.delete(id);
    }

    pub fn delta_time(&self) -> Duration {
        self.dt
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn signal_done(&mut self, _intents: &crate::intents::Intents) {
        let now = Utc::now();
        self.dt = now - self.last_tick;
        self.last_tick = now;
        self.time += 1;
    }

    pub fn insert_entity(&mut self) -> crate::model::EntityId {
        use crate::tables::SerialId;

        let res = self.next_entity;
        self.next_entity = self.next_entity.next();
        res
    }
}

impl<'a> storage::views::FromWorld<'a> for model::Time {
    fn new(w: &'a World) -> Self {
        model::Time(w.time())
    }
}

pub fn init_inmemory_storage() -> World {
    profile!("init_inmemory_storage");
    debug!("Init Storage");

    let world = World::new();

    debug!("Init Storage done");
    world
}
