mod init;
mod payload;

use caolo_sim::{self, storage::Storage};
use log::{debug, error, info};
use serde_derive::Serialize;
use std::thread;
use std::time::{Duration, Instant};

fn init() {
    #[cfg(feature = "dotenv")]
    dep_dotenv::dotenv().unwrap_or_default();

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

#[derive(Debug, Serialize)]
struct SchemaFunctionDTO<'a> {
    name: &'a str,
    description: &'a str,
    inputs: Vec<String>,
    output: &'a str,
}

fn send_schema(client: &redis::Client) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Sending schema");
    let mut con = client.get_connection()?;

    let schema = caolo_sim::api::make_import();
    let schema = schema
        .imports()
        .iter()
        .map(|import| {
            let import = &import.desc;
            SchemaFunctionDTO {
                name: import.name,
                inputs: import.inputs.iter().map(|x| x.to_string()).collect(),
                description: import.desc,
                output: import.output,
            }
        })
        .collect::<Vec<_>>();
    let js = serde_json::to_string(&schema)?;

    redis::pipe()
        .cmd("SET")
        .arg("SCHEMA")
        .arg(js)
        .query(&mut con)?;

    debug!("Sending schema done");
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
            debug!("Inserting new program {:?}", program);

            use caolo_sim::model::{ScriptComponent, ScriptId};

            let script_id = ScriptId::default(); // TODO read from users?
            let program = ScriptComponent(program);
            storage
                .scripts_table_mut::<ScriptComponent>()
                .insert_or_update(script_id, program);
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

    let tick_freq = std::env::var("TARGET_TICK_FREQUENCY_MS")
        .map(|i| i.parse::<u64>().unwrap())
        .unwrap_or(200);
    let tick_freq = Duration::from_millis(tick_freq);

    send_schema(&client).expect("Send schema");
    loop {
        let start = Instant::now();
        update_program(&mut storage, &client);
        tick(&mut storage);
        send_world(&storage, &client).expect("Sending world");
        let t = Instant::now() - start;
        let sleep_duration = tick_freq.checked_sub(t).unwrap_or(Duration::from_millis(0));
        thread::sleep(sleep_duration);
    }
}
