#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod init;
mod payload;

use caolo_sim::{self, storage::Storage};
use std::thread;
use std::time::Duration;

fn init() {
    dotenv::dotenv().unwrap_or_default(); // TODO: conf if used
    env_logger::init();
}

fn tick(storage: &mut Storage) {
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

fn send_world(storage: &Storage, client: &redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Sending world state to redis");

    let payload = payload::Payload::new(storage);
    let js = serde_json::to_string(&payload)?;

    let mut con = client.get_connection()?;

    redis::pipe()
        .cmd("SET")
        .arg("WORLD_STATE")
        .arg(js)
        .query(&mut con)?;

    debug!("Sending world state done");
    Ok(())
}

fn update_program(storage: &mut Storage, client: &redis::Client) {
    debug!("Fetching new program");
    let mut connection = client.get_connection().expect("Get redis conn");
    redis::pipe()
        .cmd("GET")
        .arg("PROGRAM")
        .cmd("DEL")
        .arg("PROGRAM")
        .ignore()
        .query(&mut connection)
        .map_err(|e| {
            error!("Failed to GET script {:?}", e);
        })
        .and_then(|program: Vec<Option<String>>| {
            program
                .get(0)
                .and_then(|program| program.clone())
                .and_then(|program| {
                    debug!("Deserializing program {:?}", program);
                    serde_json::from_str::<caolo_api::Script>(&program)
                        .map_err(|e| {
                            error!("Failed to deserialize script {:?}", e);
                        })
                        .ok()
                })
                .ok_or_else(|| ())
        })
        .map(|program| {
            debug!("Inserting new prgoram {:?}", program);

            use caolo_api::{Script, ScriptId};

            let script_id = ScriptId::default(); // TODO read from users?
            storage
                .scripts_table_mut::<Script>()
                .insert(script_id, program);
        })
        .unwrap_or(());
    debug!("Fetching new program done");
}

fn main() {
    init();
    let n_actors = std::env::var("N_ACTORS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);

    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/0".to_owned());

    let mut storage = init::init_storage(n_actors);
    let client = redis::Client::open(redis_url.as_str()).expect("Redis client");

    let sleep_duration = std::env::var("SLEEP_AFTER_TICK_MS")
        .map(|i| i.parse::<u64>().unwrap())
        .unwrap_or(200);
    loop {
        update_program(&mut storage, &client);
        tick(&mut storage);
        send_world(&storage, &client).expect("Sending world");
        thread::sleep(Duration::from_millis(sleep_duration));
    }
}
