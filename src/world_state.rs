use crate::model::world::WorldState;

use crate::PgPool;
use anyhow::Context;
use capnp::message::{ReaderOptions, TypedReader};
use capnp::serialize::try_read_message;
use redis::Commands;
use slog::{debug, error, o, Logger};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{self, Duration};

pub async fn refresh_state_job(
    pool: PgPool,
    logger: Logger,
    state: Arc<RwLock<WorldState>>,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut interval = time::interval(interval);

    loop {
        interval.tick().await;
        debug!(logger, "Reading world state");

        let new_state = load_state(pool.clone(), logger.clone()).await?;
        let mut state = state.write().unwrap();
        *state = new_state;

        debug!(logger, "Reading world state - done");
    }
}

// TODO queen tag?
pub async fn load_state(pool: PgPool, logger: Logger) -> anyhow::Result<WorldState> {
    struct Foo {
        payload: serde_json::Value,
    };

    sqlx::query_as!(
        Foo,
        r#"
        SELECT t.payload
        FROM world_output t
        ORDER BY t.created DESC
        LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .map(|Foo { payload }| WorldState(payload))
    .map_err(|err| {
        error!(logger, "Failed to read world state {:?}", err);
        err
    })
    .with_context(|| "Failed to read world state")
}
