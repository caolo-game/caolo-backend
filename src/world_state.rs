use crate::model::world::{AxialPoint, WorldState};
use crate::PgPool;
use anyhow::Context;

use slog::{debug, Logger};

use std::sync::{Arc, RwLock};
use tokio::time::{self, Duration};
use uuid::Uuid;

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

pub async fn load_state(pool: PgPool, logger: Logger) -> anyhow::Result<WorldState> {
    fn parse_axial(s: &str) -> Result<AxialPoint, &str> {
        fn _parse(s: &str) -> Option<AxialPoint> {
            let mut it = s.split(';');
            let q = it.next()?.parse().ok()?;
            let r = it.next()?.parse().ok()?;
            Some(AxialPoint { q, r })
        }
        _parse(s).ok_or(s)
    }

    struct Row {
        timestamp: i64,
        queen_tag: Uuid,
        payload: serde_json::Value,
    }
    let mut state = sqlx::query_as!(
        Row,
        r#"
        SELECT world_time AS timestamp, queen_tag, payload
        FROM world_output
        ORDER BY world_time
        DESC
        LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .with_context(|| "Failed to get state from database")?;

    debug!(
        logger,
        "Loaded state at time: {} from queen: {}", state.timestamp, state.queen_tag
    );

    let payload = state
        .payload
        .as_object_mut()
        .with_context(|| "Payload is not an object")?;

    let mut state = WorldState::default();
    state.game_config = payload["gameConfig"].take();
    state.room_properties = payload["roomProperties"].take();

    for (key, obj) in payload["bots"]
        .as_object_mut()
        .with_context(|| "`bots` is not an object")?
        .iter_mut()
    {
        let key = parse_axial(key).expect("Failed to parse key");
        let obj = obj.take();
        state
            .rooms
            .entry(key)
            .or_insert_with(Default::default)
            .bots
            .push(obj);
    }

    for (key, obj) in payload["resources"]
        .as_object_mut()
        .with_context(|| "`resources` is not an object")?
        .iter_mut()
    {
        let key = parse_axial(key).expect("Failed to parse key");
        let obj = obj.take();
        state
            .rooms
            .entry(key)
            .or_insert_with(Default::default)
            .resources
            .push(obj);
    }

    for (key, obj) in payload["structures"]
        .as_object_mut()
        .with_context(|| "`structures` is not an object")?
        .iter_mut()
    {
        let key = parse_axial(key).expect("Failed to parse key");
        let obj = obj.take();
        state
            .rooms
            .entry(key)
            .or_insert_with(Default::default)
            .structures
            .push(obj);
    }

    Ok(state)
}
