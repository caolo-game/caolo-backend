mod init;
mod input;
mod output;

use anyhow::Context;
use caolo_sim::prelude::*;
use log::{debug, error, info};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

use caolo_messages::{
    Function, RoomProperties as RoomPropertiesMsg, Schema, WorldState, WorldTerrain,
};

fn init() {
    #[cfg(feature = "dotenv")]
    dep_dotenv::dotenv().unwrap_or_default();

    pretty_env_logger::init();
}

fn tick(storage: &mut World) {
    let start = chrono::Utc::now();

    caolo_sim::forward(storage)
        .map(|_| {
            let duration = chrono::Utc::now() - start;

            info!(
                "Tick {} has been completed in {} ms",
                storage.time(),
                duration.num_milliseconds()
            );
        })
        .map_err(|e| {
            error!("Failure in forward {:?}", e);
        })
        .unwrap();
}

fn send_world(storage: &World, client: &redis::Client) -> anyhow::Result<()> {
    debug!("Sending world state");

    let bots: Vec<_> = output::build_bots(FromWorld::new(storage)).collect();

    debug!("sending {} bots", bots.len());

    let logs: Vec<_> = output::build_logs(FromWorld::new(storage)).collect();

    debug!("sending {} logs", logs.len());

    let resources: Vec<_> = output::build_resources(FromWorld::new(storage)).collect();

    debug!("sending {} resources", resources.len());

    let structures: Vec<_> = output::build_structures(FromWorld::new(storage)).collect();

    debug!("sending {} structures", structures.len());

    let world = WorldState {
        bots,
        logs,
        resources,
        structures,
    };

    let payload = rmp_serde::to_vec_named(&world)?;

    debug!("sending {} bytes", payload.len());

    let mut con = client.get_connection()?;
    redis::pipe()
        .cmd("SET")
        .arg("WORLD_STATE")
        .arg(payload)
        .query(&mut con)
        .with_context(|| "Failed to send WORLD_STATE")?;

    debug!("Sending world state done");
    Ok(())
}

#[derive(Debug, Clone, Error)]
pub enum TerrainSendFail {
    #[error("RoomProperties were not set")]
    RoomPropertiesNotSet,
}

fn send_terrain(storage: &World, client: &redis::Client) -> anyhow::Result<()> {
    let room_properties = storage
        .view::<EmptyKey, components::RoomProperties>()
        .value
        .as_ref()
        .map(|rp| RoomPropertiesMsg {
            room_radius: rp.radius,
        })
        .ok_or_else(|| TerrainSendFail::RoomPropertiesNotSet)?;

    let tiles = output::build_terrain(FromWorld::new(storage)).collect::<Vec<_>>();

    debug!("sending {} terrain", tiles.len());

    let world = WorldTerrain {
        tiles,
        room_properties,
    };

    let payload = rmp_serde::to_vec_named(&world).unwrap();
    debug!("sending {} bytes", payload.len());

    let mut con = client.get_connection()?;
    redis::pipe()
        .cmd("SET")
        .arg("WORLD_TERRAIN")
        .arg(payload)
        .query(&mut con)
        .with_context(|| "Failed to set WORLD_TERRAIN")?;

    debug!("sending terrain done");
    Ok(())
}

fn send_schema(client: &redis::Client) -> anyhow::Result<()> {
    debug!("Sending schema");
    let mut con = client.get_connection()?;

    let schema = caolo_sim::api::make_import();
    let functions = schema
        .imports()
        .iter()
        .map(|import| {
            let import = &import.desc;
            let fun = Function {
                name: import.name.to_owned(),
                description: import.description.to_owned(),
                input: import
                    .input
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
                output: import
                    .output
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
                params: import
                    .params
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            };
            fun
        })
        .collect::<Vec<_>>();

    let schema = Schema { functions };

    let payload = rmp_serde::to_vec_named(&schema).unwrap();

    redis::pipe()
        .cmd("SET")
        .arg("SCHEMA")
        .arg(payload)
        .query(&mut con)
        .with_context(|| "Failed to set SCHEMA")?;

    debug!("Sending schema done");
    Ok(())
}

fn main() {
    init();
    let _guard = std::env::var("SENTRY_URI")
        .ok()
        .map(|uri| sentry::init(uri));
    let n_actors = std::env::var("N_ACTORS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_owned());

    let mut storage = init::init_storage(n_actors);
    let client = redis::Client::open(redis_url.as_str()).expect("Redis client");

    send_terrain(&*storage.as_ref(), &client).expect("Send terrain");

    let tick_freq = std::env::var("TARGET_TICK_FREQUENCY_MS")
        .map(|i| i.parse::<u64>().unwrap())
        .unwrap_or(200);
    let tick_freq = Duration::from_millis(tick_freq);

    send_schema(&client).expect("Send schema");

    sentry::capture_message(
        "Caolo Worker initialization complete! Starting the game loop",
        sentry::Level::Info,
    );
    loop {
        let start = Instant::now();
        input::handle_messages(&mut storage, &client);
        tick(&mut storage);
        send_world(&storage, &client).expect("Sending world");
        let t = Instant::now() - start;
        let sleep_duration = tick_freq
            .checked_sub(t)
            .unwrap_or_else(|| Duration::from_millis(0));
        thread::sleep(sleep_duration);
    }
}
