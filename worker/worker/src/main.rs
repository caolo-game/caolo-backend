mod config;
mod init;
mod input;

mod protos;

use anyhow::Context;
use caolo_sim::{executor::Executor, executor::SimpleExecutor, prelude::*};
use redis::AsyncCommands;
use slog::{debug, error, info, o, warn, Drain, Logger};
use std::{
    env,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn init() {
    #[cfg(feature = "dotenv")]
    dotenv::dotenv().unwrap_or_default();
}

fn tick(logger: Logger, exc: &mut impl Executor, storage: &mut World) {
    let start = chrono::Utc::now();
    exc.forward(storage)
        .map(|_| {
            let duration = chrono::Utc::now() - start;

            info!(
                logger,
                "Tick {} has been completed in {} ms",
                storage.time(),
                duration.num_milliseconds()
            );
        })
        .expect("Failed to forward game state")
}

async fn send_schema<'a>(
    logger: Logger,
    connection: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
    queen_tag: &'a str,
) -> anyhow::Result<()> {
    debug!(logger, "Sending schema");
    let schema = caolo_sim::scripting_api::make_import();
    let imports = schema.imports();

    let basic_descs = cao_lang::compiler::card_description::get_instruction_descriptions();

    #[derive(serde::Serialize)]
    struct Card<'a> {
        name: &'a str,
        description: &'a str,
        ty: &'a str,
        input: &'a [&'a str],
        output: &'a [&'a str],
        constants: &'a [&'a str],
    }

    let msg = imports
        .iter()
        .map(|import| Card {
            name: import.desc.name,
            description: import.desc.description,
            constants: &*import.desc.constants,
            input: &*import.desc.input,
            output: &*import.desc.output,
            ty: import.desc.ty.as_str(),
        })
        .chain(basic_descs.iter().map(|card| Card {
            name: card.name,
            description: card.description,
            input: &*card.input,
            output: &*card.output,
            constants: &*card.constants,
            ty: card.ty.as_str(),
        }))
        .collect::<Vec<_>>();

    let payload = serde_json::to_value(&msg)?;

    sqlx::query!(
        r#"
    INSERT INTO scripting_schema (queen_tag, payload)
    VALUES ($1, $2)
    ON CONFLICT (queen_tag)
    DO UPDATE SET
    payload=$2
        "#,
        queen_tag,
        payload
    )
    .execute(connection)
    .await
    .with_context(|| "Failed to send schema")?;

    debug!(logger, "Sending schema done");
    Ok(())
}

/// Publish the world to {queen_tag}-world
async fn output_to_redis<'a>(
    payload: &'a serde_json::Value,
    client: &'a redis::Client,
    queen_tag: &'a str,
) -> anyhow::Result<()> {
    let payload = serde_json::to_vec(payload).unwrap();

    let mut conn = client
        .get_async_connection()
        .await
        .with_context(|| "Failed to acquire redis connection")?;

    conn.publish(format!("{}-world", queen_tag), payload)
        .await
        .with_context(|| "Failed to send world payload")?;

    Ok(())
}

async fn output_to_db<'a>(
    time: i64,
    payload: &'a serde_json::Value,
    connection: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
    queen_tag: &'a str,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO world_output (queen_tag, world_time, payload)
        VALUES ($1, $2, $3);
        "#,
        queen_tag,
        time,
        payload
    )
    .execute(connection)
    .await
    .with_context(|| "Failed to insert current world state")?;
    Ok(())
}

async fn output<'a>(
    time: i64,
    payload: &'a serde_json::Value,
    queen_tag: &'a str,
    db: impl sqlx::Executor<'a, Database = sqlx::Postgres>,
    redis: &'a redis::Client,
) -> anyhow::Result<()> {
    let db = output_to_db(time, payload, db, queen_tag);
    let red = output_to_redis(payload, redis, queen_tag);

    db.await.with_context(|| "Failed to send to database")?;
    red.await.with_context(|| "Failed to send to redis")?;
    Ok(())
}

fn main() {
    init();
    let sim_rt = caolo_sim::RuntimeGuard::new();

    let game_conf = config::GameConfig::load();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain).fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(slog_async::OverflowStrategy::Drop)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!());

    info!(logger, "Loaded game config {:?}", game_conf);

    let _sentry = env::var("SENTRY_URI")
        .ok()
        .map(|uri| {
            let options: sentry::ClientOptions = uri.as_str().into();
            let integration = sentry_slog::SlogIntegration::default();
            sentry::init(options.add_integration(integration))
        })
        .ok_or_else(|| {
            warn!(logger, "Sentry URI was not provided");
        });

    let script_chunk_size = env::var("CAO_QUEEN_SCRIPT_CHUNK_SIZE")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(1024);

    let tick_latency = Duration::from_millis(game_conf.target_tick_ms);

    info!(
        logger,
        "Loaded Queen params:\nScript chunk size: {}\nTick latency: {:?}",
        script_chunk_size,
        tick_latency
    );
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:admin@localhost:5432/caolo".to_owned());

    let db_pool = sim_rt
        .block_on(sqlx::PgPool::connect(database_url.as_str()))
        .expect("failed to connect to database");

    let tag = env::var("CAO_QUEEN_TAG").unwrap_or_else(|_| Uuid::new_v4().to_string());

    info!(logger, "Creating cao executor with tag {}", tag);
    let mut executor = SimpleExecutor;
    info!(logger, "Init storage");
    let mut storage = executor
        .initialize(
            Some(logger.clone()),
            caolo_sim::executor::GameConfig {
                world_radius: game_conf.world_radius,
                room_radius: game_conf.room_radius,
                queen_tag: tag,
                ..Default::default()
            },
        )
        .expect("Initialize executor");

    let queue_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_owned());
    let redis_client = redis::Client::open(queue_url).expect("Failed to connect to redis");

    info!(logger, "Starting with {} actors", game_conf.n_actors);

    sim_rt
        .block_on(send_schema(
            logger.clone(),
            &db_pool,
            storage.queen_tag().unwrap(),
        ))
        .expect("Failed to send schema");
    init::init_storage(logger.clone(), &mut storage, &game_conf);

    sentry::capture_message(
        format!(
            "Caolo Worker {} initialization complete! Starting the game loop",
            storage.queen_tag().unwrap()
        )
        .as_str(),
        sentry::Level::Info,
    );

    loop {
        let start = Instant::now();

        tick(logger.clone(), &mut executor, &mut storage);

        let world_json = storage.as_json();
        sim_rt
            .block_on(output(
                storage.time() as i64,
                &world_json,
                storage.queen_tag().unwrap(),
                &db_pool,
                &redis_client,
            ))
            .map_err(|err| {
                error!(logger, "Failed to send world output to storage {:?}", err);
            })
            .unwrap_or(());

        let mut sleep_duration = tick_latency
            .checked_sub(Instant::now() - start)
            .unwrap_or_else(|| Duration::from_millis(0));

        // use the sleep time to update inputs
        // this allows faster responses to clients as well as potentially spending less time on
        // inputs because handling them is built into the sleep cycle
        while sleep_duration > Duration::from_millis(0) {
            let start = Instant::now();
            sim_rt
                .block_on(input::handle_messages(
                    logger.clone(),
                    &mut storage,
                    &redis_client,
                ))
                .map_err(|err| {
                    error!(logger, "Failed to handle inputs {:?}", err);
                })
                .unwrap_or(());
            sleep_duration = sleep_duration
                .checked_sub(Instant::now() - start)
                // the idea is to sleep for half of the remaining time, then handle messages again
                .and_then(|d| d.checked_div(2))
                .unwrap_or_else(|| Duration::from_millis(0));
            std::thread::sleep(sleep_duration);
        }
    }
}
