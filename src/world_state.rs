use crate::model::world::WorldState;

use crate::parsers::*;
use crate::RedisPool;
use anyhow::Context;
use capnp::message::{ReaderOptions, TypedReader};
use capnp::serialize::try_read_message;
use redis::Commands;
use slog::{debug, error, o, Logger};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{self, Duration};

use cao_messages::world_capnp;

type InputMsg = TypedReader<capnp::serialize::OwnedSegments, world_capnp::world_state::Owned>;

pub async fn refresh_state_job(
    key: &str,
    pool: RedisPool,
    logger: Logger,
    state: Arc<RwLock<WorldState>>,
    interval: Duration,
) -> anyhow::Result<()> {
    let mut interval = time::interval(interval);

    let logger = logger.new(o!("key"=>key.to_owned()));

    loop {
        interval.tick().await;
        debug!(logger, "Reading world state");

        let new_state = load_state(key, pool.clone(), logger.clone()).await?;

        let mut state = state.write().unwrap();
        *state = new_state;

        debug!(logger, "Reading world state - done");
    }
}

pub async fn load_state(key: &str, pool: RedisPool, logger: Logger) -> anyhow::Result<WorldState> {
    let mut connection = pool.get().unwrap();
    // TODO: async pls
    connection
        .get(key)
        .with_context(|| "Failed to get state from redis")
        .and_then(|message: Vec<u8>| {
            try_read_message(
                message.as_slice(),
                ReaderOptions {
                    traversal_limit_in_words: 500_000,
                    nesting_limit: 32,
                },
            )
            .map_err(|err| {
                error!(logger, "Failed to parse capnp message {:?}", err);
                err
            })?
            .map(|x| x.into_typed())
            .with_context(|| "Failed to get typed reader")
        })
        .and_then(|state: InputMsg| {
            let state = state.get().expect("failed to get reader");
            let mut rooms = HashMap::with_capacity(1024);

            let mut bot_count = 0;
            for bot in state.reborrow().get_bots().expect("bots").iter() {
                parse_bot(&bot, &mut rooms);
                bot_count += 1;
            }

            for structure in state
                .reborrow()
                .get_structures()
                .expect("structures")
                .iter()
            {
                parse_structure(&structure, &mut rooms);
            }

            for resource in state.reborrow().get_resources().expect("resources").iter() {
                parse_resource(&resource, &mut rooms);
            }

            let mut history = HashMap::with_capacity(bot_count);

            for entry in state
                .reborrow()
                .get_script_history()
                .expect("script_history")
                .iter()
            {
                let entry = parse_script_history(&entry);
                history.insert(entry.entity_id, entry);
            }

            let mut logs = Vec::new();
            for entry in state.reborrow().get_logs().expect("logs").iter() {
                let entry = parse_log(&entry);
                logs.push(entry);
            }

            Ok(WorldState {
                rooms,
                script_history: history,
                logs,
            })
        })
        .map_err(|err| {
            error!(logger, "Failed to read world state {:?}", err);
            err
        })
        .with_context(|| "Failed to read world state")
}
