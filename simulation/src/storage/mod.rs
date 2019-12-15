mod homogenoustable;
mod macros;

pub use crate::tables::{Component, TableId, TableRow};

use crate::implement_table_type;
use crate::intents::Intent;
use crate::model::indices::EntityTime;
use crate::model::*;
use chrono::{DateTime, Duration, Utc};
use homogenoustable::HomogenousTable;
use std::any::{type_name, TypeId};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Storage {
    time: u64,
    next_entity: EntityId,
    entity_tables: HashMap<TypeId, HomogenousTable<EntityId>>,
    user_tables: HashMap<TypeId, HomogenousTable<UserId>>,
    point_tables: HashMap<TypeId, HomogenousTable<Point>>,
    scripts_tables: HashMap<TypeId, HomogenousTable<ScriptId>>,
    log_tables: HashMap<TypeId, HomogenousTable<EntityTime>>,

    last_tick: DateTime<Utc>,
    dt: Duration,
}

unsafe impl Send for Storage {}
unsafe impl Sync for Storage {}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        Self {
            time: 0,
            next_entity: EntityId(0),
            entity_tables: HashMap::new(),
            user_tables: HashMap::new(),
            point_tables: HashMap::new(),
            scripts_tables: HashMap::new(),
            log_tables: HashMap::new(),

            last_tick: Utc::now(),
            dt: Duration::zero(),
        }
    }

    pub fn users<'a>(&'a self) -> impl Iterator<Item = UserId> + 'a {
        self.user_table::<UserComponent>().iter().map(|(id, _)| id)
    }

    pub fn delta_time(&self) -> Duration {
        self.dt
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn signal_done(&mut self, _intents: &[Intent]) {
        let now = Utc::now();
        self.dt = now - self.last_tick;
        self.last_tick = now;
        self.time += 1;
    }

    pub fn insert_entity(&mut self) -> EntityId {
        let res = self.next_entity;
        self.next_entity.0 += 1;
        res
    }

    implement_table_type!(
        entity_tables,
        entity_table,
        entity_table_mut,
        add_entity_table,
        delete_entity,
        EntityId
    );

    implement_table_type!(
        user_tables,
        user_table,
        user_table_mut,
        add_user_table,
        delete_user,
        UserId
    );

    implement_table_type!(
        point_tables,
        point_table,
        point_table_mut,
        add_point_table,
        delete_point,
        Point
    );

    implement_table_type!(
        scripts_tables,
        scripts_table,
        scripts_table_mut,
        add_scripts_table,
        delete_script,
        ScriptId
    );

    implement_table_type!(
        log_tables,
        log_table,
        log_table_mut,
        add_log_table,
        delete_log,
        EntityTime
    );
}
