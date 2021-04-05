mod config;
mod input;
mod output;
mod protos;

mod command_service;
mod world_service;

use crate::protos::cao_commands::command_server::CommandServer;
use caolo_sim::{executor::Executor, executor::SimpleExecutor};
use slog::{error, info, o, Drain, Logger};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

type World = std::pin::Pin<Box<caolo_sim::prelude::World>>;

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
    let queen_tag = tag.clone();
    let mut world = executor
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

    info!(logger, "Starting with {} actors", game_conf.n_actors);

    sim_rt
        .block_on(output::send_schema(
            logger.clone(),
            &db_pool,
            queen_tag.as_str(),
        ))
        .expect("Failed to send schema");
    caolo_sim::init::init_world_entities(logger.clone(), &mut world, game_conf.n_actors as usize);

    let world = Arc::new(tokio::sync::Mutex::new(world));

    let addr = env::var("CAO_SERVICE_ADDR")
        .ok()
        .map(|x| x.parse().expect("failed to parse cao service address"))
        .unwrap_or_else(|| "[::1]:50051".parse().unwrap());

    info!(
        logger,
        "Init done. Starting the game loop. Starting the service on {:?}", addr
    );

    let server = tonic::transport::Server::builder()
        .add_service(CommandServer::new(
            crate::command_service::CommandService::new(logger.clone(), Arc::clone(&world)),
        ))
        .serve(addr);

    let game_loop = async move {
        loop {
            let start = Instant::now();
            let world_json;
            let time;
            {
                // free the world mutex at the end of this scope
                let mut world = world.lock().await;

                tick(logger.clone(), &mut executor, &mut *world);

                world_json = world.as_json();
                time = world.time() as i64;
            }
            output::send_world(
                logger.clone(),
                time,
                &world_json,
                queen_tag.as_str(),
                &db_pool,
            )
            .await
            .map_err(|err| {
                error!(logger, "Failed to send world output to storage {:?}", err);
            })
            .unwrap_or(());

            let sleep_duration = tick_latency
                .checked_sub(Instant::now() - start)
                .unwrap_or_else(|| Duration::from_millis(0));

            // use the sleep time to update clients
            tokio::time::sleep(sleep_duration).await;
        }
        // using this for a type hint
        #[allow(unreachable_code)]
        Ok::<_, ()>(())
    };

    sim_rt.block_on(async move {
        let (a, b) = futures::join!(server, game_loop);
        a.unwrap();
        b.unwrap();
    });
}
