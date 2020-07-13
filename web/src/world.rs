use crate::model::User;
use crate::RedisPool;
use caolo_messages::WorldState;
use futures::stream::StreamExt;
use futures::FutureExt;
use log::{debug, error, trace};
use redis::Commands;
use redis::RedisError;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedSender};
use warp::ws::{Message, WebSocket};

#[derive(Debug)]
struct WorldStream {
    pub hb: Instant,
    pub last_sent: Instant,
    pub pool: RedisPool,
    pub user: Option<User>,
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("RedisError {0:?}")]
    RedisError(RedisError),
}

#[derive(Debug, Error)]
pub enum WorldSendError {
    #[error("Failed to send {0:?}")]
    SendError(mpsc::error::SendError<std::result::Result<warp::filters::ws::Message, warp::Error>>),
    #[error("Failed to read world state {0:?}")]
    ReadError(ReadError),
}

impl WorldStream {
    pub fn new(user: Option<User>, pool: RedisPool) -> Self {
        Self {
            user,
            pool,
            hb: Instant::now(),
            last_sent: Instant::now(),
        }
    }
}

pub fn send(
    pool: &RedisPool,
    sender: &mut UnboundedSender<Result<Message, warp::Error>>,
) -> Result<(), WorldSendError> {
    let mut connection = pool.get().unwrap();
    let state = connection
        .get::<_, Vec<u8>>("WORLD_STATE")
        .map_err(|err| ReadError::RedisError(err))
        .map(|bytes| {
            rmp_serde::from_read_ref(bytes.as_slice()).expect("WorldState deserialization error")
        })
        .map_err(WorldSendError::ReadError)?;
    let state: WorldState = state;
    trace!("Sending world state to client");
    let mut buffer = Vec::with_capacity(512);
    serde_json::to_writer(&mut buffer, &state).expect("json serialize");
    let msg = Message::binary(buffer);
    sender.send(Ok(msg)).map_err(WorldSendError::SendError)?;
    Ok(())
}

pub async fn world_stream(ws: WebSocket, user: Option<User>, pool: RedisPool) {
    log::debug!("Starting world stream for user {:?}", user);

    let (world_ws_tx, mut world_ws_rx) = ws.split();

    let handler = WorldStream::new(user, pool.clone());

    let (mut tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(world_ws_tx).map(|result| {
        if let Err(err) = result {
            log::debug!("Websocket send error: {}", err);
        }
    }));

    tokio::task::spawn({
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                if let Err(err) = send(&pool, &mut tx) {
                    match err {
                        WorldSendError::SendError(mpsc::error::SendError(Err(err))) => {
                            error!("Failed to send world state {:?}", err)
                        }
                        _ => trace!("Failed to send world state {:?}", err),
                    }
                    break;
                }
            }

            debug!("Stopping stream");
        }
    });

    while let Some(result) = world_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Websocket error(user={:?}): {}", handler.user, err);
                break;
            }
        };
        debug!("Received message by user {:?}", msg);
    }

    log::debug!("Bye user {:?}", handler.user);
}
