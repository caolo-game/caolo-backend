use crate::model::User;
use crate::RedisPool;
use caolo_messages::WorldState;
use futures::stream::StreamExt;
use futures::FutureExt;
use redis::Commands;
use redis::RedisError;
use slog::{debug, error, info, trace, warn};
use slog::{o, Logger};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedSender};
use warp::ws::{Message, WebSocket};

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

pub fn send(
    logger: &Logger,
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
    trace!(logger, "Sending world state to client");
    let mut buffer = Vec::with_capacity(512);
    serde_json::to_writer(&mut buffer, &state).expect("json serialize");
    let msg = Message::binary(buffer);
    sender.send(Ok(msg)).map_err(WorldSendError::SendError)?;
    Ok(())
}

pub async fn world_stream(logger: Logger, ws: WebSocket, user: Option<User>, pool: RedisPool) {
    let logger = logger.new(o!("user_id" => user.as_ref().map(|u|format!("{}",u.id))));
    info!(logger, "Starting world stream");

    let (world_ws_tx, mut world_ws_rx) = ws.split();

    let (mut tx, rx) = mpsc::unbounded_channel();
    {
        let logger = logger.clone();
        tokio::task::spawn(rx.forward(world_ws_tx).map(move |result| {
            if let Err(err) = result {
                debug!(logger, "Websocket send error: {}", err);
            }
        }));
    }

    tokio::task::spawn({
        let logger = logger.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                if let Err(err) = send(&logger, &pool, &mut tx) {
                    match err {
                        WorldSendError::SendError(mpsc::error::SendError(Err(err))) => {
                            warn!(logger, "Failed to send world state {:?}", err)
                        }
                        _ => trace!(logger, "Failed to send world state {:?}", err),
                    }
                    break;
                }
            }

            debug!(logger, "Stopping stream");
        }
    });

    while let Some(result) = world_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(err) => {
                error!(logger, "Websocket error(user={:?}): {}", user, err);
                break;
            }
        };
        debug!(logger, "Received message from user {:?}", msg);
    }

    info!(logger, "Bye user {:?}", user.map(|u| u.id));
}
