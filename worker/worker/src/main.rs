mod config;
mod input;
mod output;

mod protos;

use caolo_sim::{executor::Executor, executor::SimpleExecutor, prelude::*};
use slog::{error, info, o, Drain, Logger};
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

fn main() {
    init();
    let sim_rt = caolo_sim::RuntimeGuard::new();

    let game_conf = config::GameConfig::load();

    let _sentry = env::var("SENTRY_URI")
        .ok()
        .map(|uri| sentry::init(uri.as_str()))
        .ok_or_else(|| {
            eprintln!("Sentry URI was not provided");
        });

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain).fuse();
    let drain = sentry_slog::SentryDrain::new(drain).fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(slog_async::OverflowStrategy::Drop)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!());

    info!(logger, "Loaded game config {:?}", game_conf);

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
    let logger = logger.new(o!("queen_tag" => tag.clone()));

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
        .block_on(output::send_schema(
            logger.clone(),
            &db_pool,
            storage.queen_tag().unwrap(),
        ))
        .expect("Failed to send schema");
    caolo_sim::init::init_world_entities(logger.clone(), &mut storage, game_conf.n_actors as usize);

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
            .block_on(output::send_world(
                logger.clone(),
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
                .map_err(|err| error!(logger, "Failed to handle inputs {:?}", err))
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
