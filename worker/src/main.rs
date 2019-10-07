#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use caolo_api::point::Point;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod init;
mod payload;

use caolo_engine::{self, storage::Storage};

fn init() {
    dotenv::dotenv().unwrap_or_default(); // TODO: conf if used
    env_logger::init();
}

fn tick(storage: web::Data<Mutex<Storage>>) {
    let mut storage = storage.lock().unwrap();

    let start = chrono::Utc::now();

    caolo_engine::forward(&mut storage)
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

struct WorldWs {
    storage: web::Data<Mutex<Storage>>,
    last_sent: Arc<AtomicUsize>, // last sent tick
    vision: [Point; 2],          // from to
}

impl Actor for WorldWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WorldWs {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WorldWs {
    fn new(storage: web::Data<Mutex<Storage>>) -> Self {
        let time = storage.lock().unwrap().time();
        Self {
            vision: [Point::new(-40, -20), Point::new(20, 20)],
            last_sent: Arc::new(AtomicUsize::new(time as usize)),
            storage,
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(200);

        ctx.run_interval(HEARTBEAT_INTERVAL, move |this, ctx| {
            let storage = this.storage.clone();
            let last_sent = this.last_sent.clone();
            let vision = this.vision;

            ctx.ping("");

            debug!("Updating websocket actor");

            if let Some(p) = {
                let storage = storage.lock().unwrap();
                let time = storage.time();
                if last_sent.load(Ordering::SeqCst) != time as usize {
                    last_sent.store(time as usize, Ordering::SeqCst);

                    Some(payload::Payload::new(&storage, &vision))
                } else {
                    None
                }
            } {
                // Free the mutex lock before bothering with serialization
                debug!("Sending payload to client: {:#?}", p);
                let p = serde_json::to_string(&p).unwrap();
                ctx.text(&p);
            }
        });
    }
}

fn main() {
    init();
    let n_actors = std::env::var("N_ACTORS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(8);

    let mut storage = Storage::new();
    init::init_storage(n_actors, &mut storage);

    let host = std::env::var("HOST").unwrap_or("localhost".to_owned());
    let port = std::env::var("PORT").unwrap_or("8000".to_owned());

    let storage = web::Data::new(Mutex::new(storage));

    {
        let storage = storage.clone();
        thread::Builder::new()
            .name("Worker".to_owned())
            .spawn(move || loop {
                tick(storage.clone());
                // Give the websockets a chance to read the state
                thread::sleep(std::time::Duration::from_millis(200));
            })
            .unwrap();
    }

    HttpServer::new(move || {
        App::new()
            .register_data(storage.clone())
            .service(web::resource("/").to(index))
    })
    .workers(3)
    .bind(format!("{}:{}", host, port))
    .unwrap()
    .run()
    .unwrap();
}

fn index(
    storage: web::Data<Mutex<Storage>>,
    r: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    debug!("Received connection request {:?}", r);
    ws::start(WorldWs::new(storage), &r, stream)
}
