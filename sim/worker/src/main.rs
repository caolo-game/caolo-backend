mod config;
mod input;
mod output;
mod protos;

mod command_service;
mod world_service;

use crate::protos::cao_commands::command_server::CommandServer;
use crate::protos::cao_world::world_server::WorldServer;
use caolo_sim::{executor::Executor, executor::SimpleExecutor};
use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};
use uuid::Uuid;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

type World = std::pin::Pin<Box<caolo_sim::prelude::World>>;

fn init() {
    #[cfg(feature = "dotenv")]
    dotenv::dotenv().unwrap_or_default();

    tracing_subscriber::fmt::init();
}

fn tick(exc: &mut impl Executor, storage: &mut World) {
    let start = chrono::Utc::now();
    exc.forward(storage)
        .map(|_| {
            let duration = chrono::Utc::now() - start;

            info!(
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

    info!("Loaded game config {:?}", game_conf);

    let script_chunk_size = env::var("CAO_QUEEN_SCRIPT_CHUNK_SIZE")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(1024);

    let tick_latency = Duration::from_millis(game_conf.target_tick_ms);

    info!(
        "Loaded Queen params:\nScript chunk size: {}\nTick latency: {:?}",
        script_chunk_size, tick_latency
    );
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:admin@localhost:5432/caolo".to_owned());

    let db_pool = sim_rt
        .block_on(sqlx::PgPool::connect(database_url.as_str()))
        .expect("failed to connect to database");

    let tag = env::var("CAO_QUEEN_TAG").unwrap_or_else(|_| Uuid::new_v4().to_string());
    let s = tracing::error_span!("", queen_tag = tag.as_str());
    let _e = s.enter();

    info!("Creating cao executor with tag {}", tag);
    let mut executor = SimpleExecutor;
    info!("Init storage");
    let queen_tag = tag.clone();
    let mut world = executor
        .initialize(caolo_sim::executor::GameConfig {
            world_radius: game_conf.world_radius,
            room_radius: game_conf.room_radius,
            queen_tag: tag,
            ..Default::default()
        })
        .expect("Initialize executor");

    info!("Starting with {} actors", game_conf.n_actors);

    sim_rt
        .block_on(output::send_schema(&db_pool, queen_tag.as_str()))
        .expect("Failed to send schema");
    caolo_sim::init::init_world_entities(&mut world, game_conf.n_actors as usize);

    let addr = env::var("CAO_SERVICE_ADDR")
        .ok()
        .map(|x| x.parse().expect("failed to parse cao service address"))
        .unwrap_or_else(|| "[::1]:50051".parse().unwrap());

    info!("Starting the game loop. Starting the service on {:?}", addr);

    let (outtx, _) = tokio::sync::broadcast::channel(4);
    let outpayload = Arc::new(outtx);

    let room_bounds = caolo_sim::prelude::Hexagon::from_radius(
        world
            .view::<caolo_sim::indices::ConfigKey, caolo_sim::components::RoomProperties>()
            .unwrap_value()
            .radius as i32, // world.config.room_properties.unwrap_value().radius as i32,
    );

    // TODO: if we're feeling ballsy we could just use an UnsafeView, because terrain data does not
    // change
    let terrain = world
        .view::<caolo_sim::prelude::WorldPosition, caolo_sim::prelude::TerrainComponent>()
        .iter_rooms()
        .map(|(room_id, room_terrain)| {
            (
                room_id.0,
                room_terrain.iter().map(|(_, t)| t).copied().collect(),
            )
        })
        .collect();

    let world = Arc::new(tokio::sync::Mutex::new(world));

    let server = tonic::transport::Server::builder()
        .trace_fn(|_| tracing::info_span!(""))
        .add_service(CommandServer::new(
            crate::command_service::CommandService::new(Arc::clone(&world)),
        ))
        .add_service(WorldServer::new(crate::world_service::WorldService::new(
            Arc::clone(&outpayload),
            room_bounds,
            Arc::new(terrain),
        )))
        .serve(addr);

    let game_loop = async move {
        loop {
            let start = Instant::now();
            let mut pl = world_service::Payload::default();
            {
                // free the world mutex at the end of this scope
                let mut world = world.lock().await;

                tick(&mut executor, &mut *world);

                let time = world.time();
                pl.update(time, &world);
            }

            if outpayload.receiver_count() > 0 {
                debug!("Sending world entities to subsribers");
                // while we're sending to the database, also update the outbound payload

                if outpayload.send(Arc::new(pl)).is_err() {
                    // happens if the subscribers disconnect while we prepared the payload
                    warn!("Lost all world subscribers");
                }
            }

            let sleep_duration = tick_latency
                .checked_sub(Instant::now() - start)
                .unwrap_or_else(|| Duration::from_millis(0));

            info!("Sleeping for {:?}", sleep_duration);
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
