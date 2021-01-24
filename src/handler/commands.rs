use super::ScriptError;
use crate::model::{
    world::{StructureType, WorldPosition},
    Identity, User,
};
use crate::PgPool;
use crate::RedisPool;
use anyhow::Context;
use cao_lang::compiler::{self, CompilationUnit};
use cao_messages::command_capnp::command::input_message;

use r2d2_redis::redis::Commands;
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

    let identity = match identity {
        Some(id) => id,
        None => {
            return Err(warp::reject::custom(CommandError::Unauthorized));
        }
    };

    let msg_id = uuid::Uuid::new_v4();

    let mut capmsg = capnp::message::Builder::new_default();
    {
        let mut message = capmsg.init_root::<input_message::Builder>();
        let mut root = message.reborrow().init_place_structure();

        let mut owner = root.reborrow().init_owner();
        owner.set_data(&identity.id.as_bytes()[..]);

        init_world_pos(&payload.position, &mut root.reborrow().init_position());

        match payload.ty {
            StructureType::Spawn => {
                root.set_ty(cao_messages::command_capnp::StructureType::Spawn);
            }
        }

        let mut id = message.reborrow().init_message_id();
        id.set_data(&msg_id.as_bytes()[..]);
    }

    send_command_to_worker(logger, msg_id, capmsg, cache)
        .await
        .map(|_| with_status(warp::reply(), StatusCode::NO_CONTENT))
        .map_err(warp::reject::custom)
}

pub async fn send_command_to_worker<A>(
    logger: Logger,
    msg_id: Uuid,
    msg: capnp::message::Builder<A>,
    cache: RedisPool,
) -> Result<(), CommandError>
where
    A: capnp::message::Allocator,
{
    use cao_messages::command_capnp::command_result;
    use capnp::message::{ReaderOptions, TypedReader};
    use capnp::serialize::try_read_message;

    let mut payload = Vec::with_capacity(5_000);
    capnp::serialize::write_message(&mut payload, &msg).expect("Failed to serialize msg");
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
    for i in 0..20 {
        // getting a response may take a while, give other threads a chance to do some work
        // TODO: read the expected game frequency from the game config
        // inrementally wait longer and longer...
        let wait_for = Duration::from_millis(50 * i);
        delay_for(wait_for).await;
        let cache = cache.clone();

        // get a new connection in each tick to free it at the end and let other threads access
        // this connection while we wait
        let mut conn = cache.get().map_err(|err| {
            error!(logger, "Failed to get redis conn {:?}", err);
            CommandError::Internal
        })?;

        type ResponseMsg = TypedReader<capnp::serialize::OwnedSegments, command_result::Owned>;

        match conn
            .get::<_, Option<Vec<u8>>>(&msg_id)
            .map::<Option<ResponseMsg>, _>(|message| {
                message.and_then(|message| {
                    try_read_message(
                        message.as_slice(),
                        ReaderOptions {
                            traversal_limit_in_words: 512,
                            nesting_limit: 64,
                        },
                    )
                    .map_err(|err| {
                        error!(logger, "Failed to parse capnp message {:?}", err);
                    })
                    .ok()?
                    .map(|x| x.into_typed())
                })
            }) {
            Ok(None) => {
                //retry
                continue;
            }
            Ok(Some(msg)) => {
                let msg = match msg.get() {
                    Ok(x) => x,
                    Err(err) => {
                        error!(logger, "Failed to get message, {:?}", err);
                        return Err(CommandError::Internal);
                    }
                };
                if msg.has_error() {
                    use cao_messages::command_capnp::command_result::Which;
                    match msg.which() {
                        Ok(Which::Error(Ok(err))) => {
                            warn!(logger, "Failed to execute command, {}", err);
                            return Err(CommandError::ExecutionError(err.to_owned()));
                        }
                        Ok(Which::Error(Err(err))) => {
                            error!(logger, "Failed to get command error, {}", err);
                            return Err(CommandError::Internal);
                        }
                        Ok(_) => {
                            return Ok(());
                        }
                        Err(err) => {
                            error!(logger, "Failed to get result variant {:?}", err);
                            return Err(CommandError::Internal);
                        }
                    }
                }
                return Ok(());
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
    let mut capmsg = capnp::message::Builder::new_default();
    {
        let mut message = capmsg.init_root::<input_message::Builder>();
        let mut root = message.reborrow().init_set_default_script();

        let mut owner = root.reborrow().init_user_id();
        owner.set_data(&identity.id.as_bytes()[..]);
        let mut script = root.reborrow().init_user_id();
        script.set_data(&script_id.as_bytes()[..]);

        let mut id = message.reborrow().init_message_id();
        id.set_data(&msg_id.as_bytes()[..]);
    }
    send_command_to_worker(logger, msg_id, capmsg, cache)
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
        }
    }

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
        owner_id: Option<Uuid>,
    }

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

    let _program = compiler::compile(None, payload.cu.clone(), None).map_err(|err| {
        debug!(logger, "compilation failure {:?}", err);
        warp::reject::custom(ScriptError::CompileError(err))
    })?;

    let QueryRes {
        id: script_id,
        owner_id: user_id,
    } = query
        .await
        .map_err(log_error!())
        .with_context(|| "failed to insert the program")
        .map_err(ScriptError::InternalError)
        .map_err(warp::reject::custom)?;

    let msg_id = uuid::Uuid::new_v4();

    let mut capmsg = capnp::message::Builder::new_default();
    {
        let mut message = capmsg.init_root::<input_message::Builder>();
        let mut id = message.reborrow().init_message_id();
        id.set_data(&msg_id.as_bytes()[..]);

        let mut root = message.reborrow().init_update_script();
        if let Some(user_id) = user_id {
            let mut owner = root.reborrow().init_user_id();
            owner.set_data(&user_id.as_bytes()[..]);
        }

        let mut script_id_msg = root.reborrow().init_script_id();
        script_id_msg.set_data(&script_id.as_bytes()[..]);

        let mut script_msg = root.reborrow().init_compilation_unit();
        let mut cu = script_msg.reborrow().init_compilation_unit();
        cu.set_value(
            serde_json::to_vec(&payload.cu)
                .expect("failed to serialize CU")
                .as_slice(),
        );
        let mut script_ver = script_msg.init_verified_by();
        script_ver.set_major(cao_lang::version::MAJOR);
        script_ver.set_minor(cao_lang::version::MINOR);
        script_ver.set_patch(cao_lang::version::PATCH);
    }

    send_command_to_worker(logger.clone(), msg_id, capmsg, cache)
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

fn init_world_pos(
    world_pos: &WorldPosition,
    builder: &mut cao_messages::point_capnp::world_position::Builder,
) {
    let mut room = builder.reborrow().init_room();
    room.set_q(world_pos.room.q);
    room.set_r(world_pos.room.r);

    let mut room_pos = builder.reborrow().init_room_pos();
    room_pos.set_q(world_pos.room_pos.q);
    room_pos.set_r(world_pos.room_pos.r);
}
