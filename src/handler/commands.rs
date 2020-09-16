use crate::model::User;
use crate::RedisPool;
use cao_messages::{
    command::{CommandResult, PlaceStructureCommand},
    InputMsg, InputPayload, StructureType, WorldPosition,
};
use redis::Commands;
use serde::Deserialize;
use slog::{debug, error, warn, Logger};
use thiserror::Error;
use tokio::time::{delay_for, Duration};
use warp::http::StatusCode;
use warp::reply::with_status;

#[derive(Error, Debug, Clone)]
pub enum CommandError {
    #[error("Log in to send commands!")]
    Unauthorized,
    #[error("Internal error happened while processing the command!")]
    Internal,
    #[error("Internal error happened while processing the command, request timed out!")]
    Timeout,
    #[error("Failed to execute command: {0}")]
    ExecutionError(String),
}
impl warp::reject::Reject for CommandError {}
impl CommandError {
    pub fn status(&self) -> StatusCode {
        match self {
            CommandError::Unauthorized => StatusCode::UNAUTHORIZED,
            CommandError::Internal | CommandError::Timeout => StatusCode::INTERNAL_SERVER_ERROR,
            CommandError::ExecutionError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlaceStructureCommandPayload {
    pub position: WorldPosition,
    pub ty: StructureType,
}

pub async fn place_structure(
    logger: Logger,
    identity: Option<User>,
    cache: RedisPool,
    payload: PlaceStructureCommandPayload,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!(
        logger,
        "Place structure command {:?} {:?}", identity, payload
    );
    if identity.is_none() {
        return Err(warp::reject::custom(CommandError::Unauthorized));
    }

    let identity = identity.unwrap();

    let command = PlaceStructureCommand {
        owner: identity.id,
        position: payload.position,
        ty: payload.ty,
    };

    let payload = InputPayload::PlaceStructure(command);

    let msg_id = uuid::Uuid::new_v4();

    let payload = InputMsg { msg_id, payload };
    let payload = rmp_serde::to_vec_named(&payload).expect("Failed to serialize inputmsg");

    {
        let cache = cache.clone();
        let l = logger.clone();
        let _: () = tokio::task::spawn_blocking(move || {
            let logger = l;
            let mut conn = cache.get().map_err(|err| {
                error!(logger, "Failed to get redis conn {:?}", err);
                CommandError::Internal
            })?;

            conn.lpush("INPUTS", payload).map_err(|err| {
                error!(logger, "Failed to push input {:?}", err);
                CommandError::Internal
            })?;
            Ok(())
        })
        .await
        .map_err(|err| {
            error!(logger, "Failed to send command {:?}", err);
            CommandError::Internal
        })
        .and_then(|x| x) // unwrap the inner error
        .map_err(warp::reject::custom)?;
    }

    let msg_id = format!("{}", msg_id);

    // retry loop
    for _ in 0..40_i32 {
        // getting a response may take a while, give other threads a chance to do some work
        // TODO: read the expected game frequency from the game config
        let wait_for = Duration::from_millis(50);
        delay_for(wait_for).await;
        let cache = cache.clone();

        // get a new connection in each tick to free it at the end and let other threads access
        // this connection while we wait
        let mut conn = cache
            .get()
            .map_err(|err| {
                error!(logger, "Failed to get redis conn {:?}", err);
                CommandError::Internal
            })
            .map_err(warp::reject::custom)?;

        match conn
            .get::<_, Option<Vec<u8>>>(&msg_id)
            .map::<Option<CommandResult>, _>(|message| {
                message.map(|message| {
                    let res: CommandResult = rmp_serde::from_read_ref(message.as_slice())
                        .expect("Failed to deserialize message");
                    res
                })
            }) {
            Ok(None) => {
                //retry
                continue;
            }
            Ok(Some(CommandResult::Ok)) => {
                // done
                return Ok(with_status(warp::reply(), StatusCode::NO_CONTENT));
            }
            Ok(Some(CommandResult::Error(err))) => {
                warn!(logger, "Failed to execute command, {}", err);
                return Err(warp::reject::custom(CommandError::ExecutionError(err)));
            }
            Err(err) => {
                error!(logger, "Failed to get response, {:?}", err);
                return Err(warp::reject::custom(CommandError::Internal));
            }
        }
    }
    Err(warp::reject::custom(CommandError::Timeout))
}
