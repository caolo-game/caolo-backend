use super::ScriptError;
use crate::model::Identity;
use crate::model::User;
use crate::PgPool;
use crate::RedisPool;
use anyhow::Context;
use cao_lang::compiler::{self, CompilationUnit};
use cao_messages::{
    command::{CommandResult, PlaceStructureCommand, SetDefaultScriptCommand, UpdateScriptCommand},
    CompiledScript, InputMsg, InputPayload, Label, StructureType, WorldPosition,
};
use redis::Commands;
use serde::Deserialize;
use slog::{debug, error, warn, Logger};
use thiserror::Error;
use tokio::time::{delay_for, Duration};
use uuid::Uuid;
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

    send_command_to_worker(logger, payload, cache)
        .await
        .map(|_| with_status(warp::reply(), StatusCode::NO_CONTENT))
        .map_err(warp::reject::custom)
}

pub async fn send_command_to_worker(
    logger: Logger,
    payload: InputMsg,
    cache: RedisPool,
) -> Result<(), CommandError> {
    let msg_id = payload.msg_id;
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
        .and_then(|x| x)?; // unwrap the inner error
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
        let mut conn = cache.get().map_err(|err| {
            error!(logger, "Failed to get redis conn {:?}", err);
            CommandError::Internal
        })?;

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
                return Ok(());
            }
            Ok(Some(CommandResult::Error(err))) => {
                warn!(logger, "Failed to execute command, {}", err);
                return Err(CommandError::ExecutionError(err));
            }
            Err(err) => {
                error!(logger, "Failed to get response, {:?}", err);
                return Err(CommandError::Internal);
            }
        }
    }
    Err(CommandError::Timeout)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptIdPayload {
    pub script_id: Uuid,
}

pub async fn set_default_script(
    logger: Logger,
    identity: Option<User>,
    ScriptIdPayload { script_id }: ScriptIdPayload,
    db: PgPool,
    cache: RedisPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let identity = identity.ok_or_else(|| warp::reject::custom(ScriptError::Unauthorized))?;

    // TODO: save the default script in DB as well...

    let exists = sqlx::query!(
        r#"
        SELECT COUNT(*)
        FROM user_script
        WHERE id=$1
        "#,
        script_id
    )
    .fetch_one(&db)
    .await
    .expect("Failed to query user_script");

    let exists = exists.count.map(|x| x != 0).unwrap_or(false);
    if !exists {
        return Err(warp::reject::not_found());
    }

    let msg_id = uuid::Uuid::new_v4();
    let payload = InputMsg {
        msg_id,
        payload: InputPayload::SetDefaultScript(SetDefaultScriptCommand {
            script_id: script_id,
            user_id: identity.id,
        }),
    };
    send_command_to_worker(logger, payload, cache)
        .await
        .map(|_| with_status(warp::reply(), StatusCode::NO_CONTENT))
        .map_err(warp::reject::custom)
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveScriptPayload {
    pub name: String,
    pub cu: CompilationUnit,
}

pub async fn commit(
    logger: Logger,
    identity: Option<Identity>,
    payload: SaveScriptPayload,
    db: PgPool,
    cache: RedisPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    macro_rules! log_error {
        () => {
            |arg| {
                error!(logger, "Error in commit {:?}", arg);
                arg
            }
        };
    };

    let mut tx = db
        .begin()
        .await
        .map_err(log_error!())
        .with_context(|| "Failed to begin transaction")
        .map_err(ScriptError::InternalError)
        .map_err(warp::reject::custom)?;

    let identity = identity.ok_or_else(|| warp::reject::custom(ScriptError::Unauthorized))?;

    struct QueryRes {
        /// script_id
        id: Uuid,
        owner_id: Uuid,
    };

    let query = {
        let name = payload.name.as_str();
        let payload =
            serde_json::to_value(&payload.cu).expect("failed to serialize CompilationUnit");
        let owner_id = identity.user_id;
        sqlx::query_file_as!(
            QueryRes,
            "src/handler/commit_script.sql",
            payload,
            name,
            owner_id,
        )
        .fetch_one(&mut tx)
    };

    let program = compiler::compile(None, payload.cu).map_err(|err| {
        debug!(logger, "compilation failure {:?}", err);
        warp::reject::custom(ScriptError::CompileError(err))
    })?;

    // map cao_lang script to cao_messages script
    let compiled_script = CompiledScript {
        bytecode: program.bytecode,
        labels: program
            .labels
            .into_iter()
            .map(|(key, cao_lang::Label { block, myself })| (key, Label { block, myself }))
            .collect(),
    };

    let QueryRes {
        id: script_id,
        owner_id: user_id,
    } = query
        .await
        .map_err(log_error!())
        .with_context(|| "failed to insert the program")
        .map_err(ScriptError::InternalError)
        .map_err(warp::reject::custom)?;

    let msg = UpdateScriptCommand {
        script_id,
        user_id,
        compiled_script,
    };

    let msg_id = uuid::Uuid::new_v4();

    send_command_to_worker(
        logger.clone(),
        InputMsg {
            msg_id,
            payload: InputPayload::UpdateScript(msg),
        },
        cache,
    )
    .await
    .map_err(ScriptError::CommandError)
    .map_err(warp::reject::custom)?;

    tx.commit()
        .await
        .with_context(|| "tx commit")
        .map_err(ScriptError::InternalError)
        .map_err(warp::reject::custom)?;

    let result = serde_json::json!({ "scriptId": script_id });
    let result = warp::reply::json(&result);
    Ok(result)
}
