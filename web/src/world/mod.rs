use crate::protos::world::WorldState;
use crate::RedisPool;
use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::web::{self, HttpRequest};
use actix_web::{get, Responder};
use actix_web_actors::ws;
use failure::Fail;
use log::{debug, error, warn};
use protobuf::{parse_from_bytes, ProtobufError};
use redis::Commands;
use redis::RedisError;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct WorldStream {
    pub hb: Instant,
    pub last_sent: Instant,
    pub pool: Arc<RedisPool>,
    pub buffer: Vec<u8>,
}

#[derive(Debug, Fail)]
enum ReadError {
    #[fail(display = "RedisError {:?}", 0)]
    RedisError(RedisError),
    #[fail(display = "ProtoError {:?}", 0)]
    ProtoError(ProtobufError),
}

impl WorldStream {
    pub fn new(pool: Arc<RedisPool>) -> Self {
        Self {
            pool,
            hb: Instant::now(),
            last_sent: Instant::now(),
            buffer: Vec::with_capacity(512),
        }
    }

    fn start_stream(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_millis(1000), |act, ctx| {
            // check client heartbeats
            let now = Instant::now();
            if now.duration_since(act.hb) > Duration::from_secs(10) {
                // heartbeat timed out
                log::debug!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
        ctx.run_interval(Duration::from_millis(500), |act, ctx| {
            let mut connection = act.pool.get().expect("get redis connection");
            match connection
                .get::<_, Vec<u8>>("WORLD_STATE")
                .map_err(ReadError::RedisError)
                .and_then(|bytes| {
                    parse_from_bytes::<WorldState>(bytes.as_slice()).map_err(ReadError::ProtoError)
                }) {
                Ok(state) => {
                    debug!("Sending world state to client");
                    let mut buffer = Vec::with_capacity(512);
                    serde_json::to_writer(&mut buffer, &state).expect("json serialize");
                    ctx.binary(buffer);
                }
                Err(e) => {
                    error!("Failed to get world state {:?}", e);
                }
            }
        });
    }
}

impl Actor for WorldStream {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("WorldStream actor is starting {:?}", self);
        self.start_stream(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WorldStream {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Err(e) => {
                warn!("WorldStream handler failed {:?}", e);
            }
            _ => {}
        }
    }
}

#[get("/world")]
pub async fn world_stream(
    req: HttpRequest,
    pool: web::Data<RedisPool>,
    stream: web::Payload,
) -> impl Responder {
    let pool = pool.into_inner();
    ws::start(WorldStream::new(pool), &req, stream)
}
