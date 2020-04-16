pub use self::store_impl::*;

use super::storage;
use crate::intents::Intents;
use crate::model::components::*;
use crate::model::geometry::Point;
use crate::model::*;
use crate::profile;
use crate::storage::views::{UnsafeView, View};
use crate::tables::{Component, TableId};
use chrono::{DateTime, Duration, Utc};
use serde_derive::Serialize;
use std::pin::Pin;

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
    key EntityId, table PathCacheComponent = entitypathcache,

    key EntityTime, table LogEntry = timelog,

    key UserId, table UserComponent = useruser,

    key Point, table TerrainComponent = pointterrain,
    key Point, table EntityComponent = pointentity,

    key ScriptId, table ScriptComponent = scriptscript
);

#[derive(Debug, Serialize)]
pub struct World {
    pub store: Storage,

    pub time: u64,
    pub next_entity: crate::model::EntityId,
    pub last_tick: DateTime<Utc>,
    #[serde(skip)]
    pub dt: Duration,
}

impl<Id: TableId, C: Component<Id>> storage::HasTable<Id, C> for World
where
    Storage: storage::HasTable<Id, C>,
{
    fn view(&self) -> View<Id, C> {
        self.store.view()
    }

    fn unsafe_view(&mut self) -> UnsafeView<Id, C> {
        self.store.unsafe_view()
    }
}

pub fn init_inmemory_storage() -> Pin<Box<World>> {
    profile!("init_inmemory_storage");
    debug!("Init Storage");

    let world = World::new();
    let world = Box::pin(world);

    debug!("Init Storage done");
    world
}

unsafe impl Send for World {}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let store = Storage::default();
        Self {
            time: 0,
            store,
            last_tick: Utc::now(),
            next_entity: crate::model::EntityId::default(),
            dt: Duration::zero(),
        }
    }

    pub fn view<Id: TableId, C: Component<Id>>(&self) -> View<Id, C>
    where
        Storage: storage::HasTable<Id, C>,
    {
        (&self.store as &dyn storage::HasTable<Id, C>).view()
    }

    pub fn unsafe_view<Id: TableId, C: Component<Id>>(&mut self) -> UnsafeView<Id, C>
    where
        Storage: storage::HasTable<Id, C>,
    {
        (&mut self.store as &mut dyn storage::HasTable<Id, C>).unsafe_view()
    }

    pub fn delete<Id: TableId>(&mut self, id: &Id)
    where
        Storage: storage::Epic<Id>,
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

    pub fn signal_done(&mut self, _intents: &Intents) {
        let now = Utc::now();
        self.dt = now - self.last_tick;
        self.last_tick = now;
        self.time += 1;
    }

    pub fn insert_entity(&mut self) -> EntityId {
        use crate::tables::SerialId;

        let res = self.next_entity;
        self.next_entity = self.next_entity.next();
        res
    }
}

impl<'a> storage::views::FromWorld<'a> for Time {
    fn new(w: &'a World) -> Self {
        Time(w.time())
    }
}