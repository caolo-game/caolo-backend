use crate::config::Config;
use crate::model::{Identity, JWKS};
use crate::RedisPool;
use caolo_messages::{ClientMessage, WorldState};
use futures::stream::StreamExt;
use futures::FutureExt;
use redis::Commands;
use redis::RedisError;
use slog::{debug, error, info, o, trace, warn, Logger};
use std::sync::{Arc, Mutex};
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

fn send(
    ctx: &Context,
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
    trace!(ctx.logger, "Sending world state to client");
    let mut buffer = Vec::with_capacity(512);
    serde_json::to_writer(&mut buffer, &state).expect("json serialize");
    let msg = Message::binary(buffer);
    sender.send(Ok(msg)).map_err(WorldSendError::SendError)?;
    Ok(())
}

#[derive(Debug)]
struct Context {
    logger: Logger,
    user: Option<Identity>,
    done: bool,
}

pub async fn world_stream(
    logger: Logger,
    ws: WebSocket,
    user: Option<Identity>,
    pool: RedisPool,
    jwks: &JWKS,
    config: Arc<Config>,
) {
    info!(logger, "Starting world stream");
    let ctx = Arc::new(Mutex::new(Context {
        logger,
        user,
        done: false,
    }));

    let (world_ws_tx, mut world_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    {
        let ctx = Arc::clone(&ctx);
        tokio::task::spawn(rx.forward(world_ws_tx).map(move |result| {
            if let Err(err) = result {
                let ctx = ctx.lock().unwrap();
                if !ctx.done {
                    let logger = &ctx.logger;
                    debug!(logger, "Websocket send error: {}", err);
                }
            }
        }));
    }

    tokio::task::spawn({
        let ctx = Arc::clone(&ctx);
        let mut tx = tx.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                let ctx = ctx.lock().unwrap();
                if ctx.done {
                    break;
                }
                let logger = &ctx.logger;
                if let Err(err) = send(&*ctx, &pool, &mut tx) {
                    match err {
                        WorldSendError::SendError(mpsc::error::SendError(Err(err))) => {
                            warn!(logger, "Failed to send world state {:?}", err)
                        }
                        WorldSendError::SendError(mpsc::error::SendError(Ok(_))) => {
                            trace!(logger, "Failed to send world state")
                        }
                        _ => trace!(logger, "Failed to send world state {:?}", err),
                    }
                    break;
                }
            }

            let logger = &ctx.lock().unwrap().logger;
            debug!(logger, "Stopping stream");
        }
    });

    while let Some(result) = world_ws_rx.next().await {
        match result {
            Ok(msg) if msg.is_text() => {
                let msg = match serde_json::from_str(msg.to_str().unwrap()) {
                    Ok(msg) => msg,
                    Err(err) => {
                        let ctx = ctx.lock().unwrap();
                        warn!(
                            ctx.logger,
                            "Failed to deserialize client message {:?}\n{:?}", msg, err
                        );
                        continue;
                    }
                };
                let mut ctx = ctx.lock().unwrap();
                debug!(ctx.logger, "Received message from user {:?}", msg);
                match msg {
                    ClientMessage::AuthToken(token) => {
                        ctx.user = Identity::validated_id(
                            &ctx.logger,
                            config.as_ref(),
                            token.as_str(),
                            jwks,
                        )
                        .or_else(|| {
                            warn!(ctx.logger, "Failed to validate auth token");
                            None
                        });
                        if let Some(ref user) = ctx.user {
                            ctx.logger = ctx.logger.new(o!("user_id" => user.user_id.clone()));
                        }
                    }
                }
            }
            Ok(msg) if msg.is_pong() || msg.is_ping() || msg.is_close() => {}
            Ok(msg) => {
                let ctx = ctx.lock().unwrap();
                warn!(ctx.logger, "Received unexpected message {:?}", msg)
            }
            Err(err) => {
                let ctx = ctx.lock().unwrap();
                warn!(ctx.logger, "Websocket error: {}", err);
                break;
            }
        }
    }

    let mut ctx = ctx.lock().unwrap();
    ctx.done = true;
    info!(ctx.logger, "Bye user");
}
