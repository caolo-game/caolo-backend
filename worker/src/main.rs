// for protobuf
#[macro_use]
extern crate serde_derive;

mod init;
mod input;
mod output;
mod protos;

use caolo_sim::prelude::*;
use log::{debug, error, info};
use protobuf::{Message, RepeatedField};
use protos::schema::{Function as SchemaFunctionDTO, Schema as SchemaMessage};
use std::thread;
use std::time::{Duration, Instant};

fn init() {
    #[cfg(feature = "dotenv")]
    dep_dotenv::dotenv().unwrap_or_default();

    env_logger::init();
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

fn send_world(storage: &World, client: &redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    use protos::world::WorldState;

    debug!("Sending world state");

    let mut world = WorldState::new();
    for bot in output::build_bots(FromWorld::new(storage)) {
        world.mut_bots().push(bot);
    }

    debug!("sending {} bots", world.get_bots().len());

    for log in output::build_logs(FromWorld::new(storage)) {
        world.mut_logs().push(log);
    }

    debug!("sending {} logs", world.get_logs().len());

    for resource in output::build_resources(FromWorld::new(storage)) {
        world.mut_resources().push(resource);
    }

    debug!("sending {} resources", world.get_resources().len());

    for structure in output::build_structures(FromWorld::new(storage)) {
        world.mut_structures().push(structure);
    }

    debug!("sending {} structures", world.get_structures().len());

    let payload = world.write_to_bytes()?;

    debug!("sending {} bytes", payload.len());

    let mut con = client.get_connection()?;
    redis::pipe()
        .cmd("SET")
        .arg("WORLD_STATE")
        .arg(payload)
        .query(&mut con)?;

    debug!("Sending world state done");
    Ok(())
}

fn send_terrain(storage: &World, client: &redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    use protos::world::WorldTerrain;

    let mut world = WorldTerrain::new();

    for tile in output::build_terrain(FromWorld::new(storage)) {
        world.mut_tiles().push(tile);
    }

    debug!("sending {} terrain", world.get_tiles().len());
    let payload = world.write_to_bytes()?;
    debug!("sending {} bytes", payload.len());

    let mut con = client.get_connection()?;
    redis::pipe()
        .cmd("SET")
        .arg("WORLD_TERRAIN")
        .arg(payload)
        .query(&mut con)?;

    debug!("sending terrain done");
    Ok(())
}

fn send_schema(client: &redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Sending schema");
    let mut con = client.get_connection()?;

    let schema = caolo_sim::api::make_import();
    let imports = schema
        .imports()
        .iter()
        .map(|import| {
            let import = &import.desc;
            let mut fun = SchemaFunctionDTO::new();
            fun.set_name(import.name.to_owned());
            fun.set_description(import.description.to_owned());
            fun.set_input(RepeatedField::from_ref(
                import
                    .input
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            ));
            fun.set_params(RepeatedField::from_ref(
                import
                    .params
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            ));
            fun.set_output(RepeatedField::from_ref(
                import
                    .output
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>(),
            ));
            fun
        })
        .collect::<Vec<_>>();

    let mut schema = SchemaMessage::new();
    schema.set_functions(RepeatedField::from_vec(imports));

    let payload = schema.write_to_bytes()?;

    redis::pipe()
        .cmd("SET")
        .arg("SCHEMA")
        .arg(payload)
        .query(&mut con)?;

    debug!("Sending schema done");
    Ok(())
}

fn main() {
    init();
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
