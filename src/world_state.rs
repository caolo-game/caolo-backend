use crate::model::world::WorldState;

use crate::PgPool;
use anyhow::Context;
use slog::{debug, error, Logger};
use tokio::time::{self, Duration};

#[derive(Clone)]
pub struct SharedState(pub left_right::ReadHandle<WorldState>);

unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}

pub struct StateWriter(pub left_right::WriteHandle<WorldState, WorldStateOps>);

pub enum WorldStateOps {
    Set(WorldState),
}

impl left_right::Absorb<WorldStateOps> for WorldState {
    fn absorb_first(&mut self, operation: &mut WorldStateOps, _other: &Self) {
        match operation {
            WorldStateOps::Set(w) => self.clone_from(w),
        }
    }

    fn absorb_second(&mut self, operation: WorldStateOps, _other: &Self) {
        match operation {
            WorldStateOps::Set(w) => *self = w,
        }
    }
}

pub fn init_state() -> (StateWriter, SharedState) {
    let (w, r) = left_right::new::<WorldState, WorldStateOps>();
    (StateWriter(w), SharedState(r))
}

pub async fn refresh_state_job(
    pool: PgPool,
    logger: Logger,
    mut state: StateWriter,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut interval = time::interval(interval);

    loop {
        interval.tick().await;
        debug!(logger, "Reading world state");

        let new_state = load_state(pool.clone(), logger.clone()).await?;
        state.0.append(WorldStateOps::Set(new_state));
        state.0.publish();

        debug!(logger, "Reading world state - done");
    }
}

// TODO queen tag?
pub async fn load_state(pool: PgPool, logger: Logger) -> anyhow::Result<WorldState> {
    struct Foo {
        world_time: i64,
        payload: serde_json::Value,
    }

    sqlx::query_as!(
        Foo,
        r#"
        SELECT t.world_time, t.payload
        FROM world_output t
        ORDER BY t.created DESC
        LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .map(
        |Foo {
             world_time,
             payload,
         }| WorldState {
            time: world_time,
            payload,
        },
    )
    .map_err(|err| {
        error!(logger, "Failed to read world state {:?}", err);
        err
    })
    .with_context(|| "Failed to read world state")
}
