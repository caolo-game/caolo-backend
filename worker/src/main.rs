#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod init;
mod payload;

use std::thread;
use std::time::Duration;
use caolo_engine::{self, storage::Storage};

fn init() {
    dotenv::dotenv().unwrap_or_default(); // TODO: conf if used
    env_logger::init();
}

fn tick(storage: &mut Storage) {
    let start = chrono::Utc::now();

    caolo_engine::forward(storage)
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

fn main() {
    init();
    let n_actors = std::env::var("N_ACTORS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);

    let redis_url = std::env::var("REDIS_URL").unwrap_or("redis://localhost:6379/0".to_owned());

    let mut storage = init::init_storage(n_actors);
    let connection = redis::Client::open(redis_url.as_str()).expect("Redis connection");

    loop {
        tick(&mut storage);
        send_world(&storage, &connection).expect("Sending world");
        thread::sleep(Duration::from_millis(200));
    }
}
