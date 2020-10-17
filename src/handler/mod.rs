mod commands;
mod rooms;
mod user;

pub use commands::*;
pub use rooms::*;
pub use user::*;

use crate::model::script::{Card, Schema};
use crate::model::world::AxialPoint;
use crate::model::{Identity, ScriptEntity, ScriptMetadata};
use crate::PgPool;
use crate::RedisPool;
use crate::SharedState;
use anyhow::Context;
use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::compiler::{self, CompilationUnit};
use redis::Commands;
use slog::{debug, error, trace, Logger};
use std::convert::Infallible;
use thiserror::Error;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn get_bot_history(
    _logger: Logger,
    entity_id: u32,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: authorize; users may only inspect their own bots
    let history = &state.read().unwrap().script_history;
    history
        .get(&entity_id)
        .map(|hist| warp::reply::json(hist))
        .ok_or_else(|| warp::reject::not_found())
}

pub async fn schema(logger: Logger, cache: RedisPool) -> Result<impl warp::Reply, Infallible> {
    use crate::parsers::parse_function_desc;
    use capnp::message::{ReaderOptions, TypedReader};
    use capnp::serialize::try_read_message;

    type InputMsg =
        TypedReader<capnp::serialize::OwnedSegments, cao_messages::script_capnp::schema::Owned>;

    let mut conn = cache.get().expect("failed to aquire cache connection");

    let basic_schema = get_instruction_descriptions();

    let schema: InputMsg = conn
        .get("SCHEMA")
        .with_context(|| "failed to read schema")
        .and_then(|message: Vec<u8>| {
            try_read_message(
                message.as_slice(),
                ReaderOptions {
                    traversal_limit_in_words: 512,
                    nesting_limit: 64,
                },
            )
            .map_err(|err| {
                error!(logger, "Failed to parse capnp message {:?}", err);
                err
            })?
            .map(|x| x.into_typed())
            .with_context(|| "Failed to get typed reader")
        })
        .expect("Failed to read schema");

    let schema = schema.get().unwrap();
    let cards = schema.get_cards().expect("schema.cards");

    let cards = cards
        .iter()
        .map(|fun| parse_function_desc(fun))
        .collect::<Vec<_>>();

    let mut schema = Schema { cards };
    schema.cards.extend(basic_schema.iter().map(|item| {
        Card::from_str_parts(
            item.name,
            item.description,
            item.ty.clone(),
            item.input.as_ref(),
            item.output.as_ref(),
            item.constants.as_ref(),
        )
    }));
    let resp = with_status(warp::reply::json(&schema), StatusCode::OK);
    Ok(resp)
}

pub async fn get_sim_config(cache: RedisPool) -> Result<impl warp::Reply, Infallible> {
    let mut conn = cache.get().expect("failed to aquire cache connection");

    let conf: serde_json::Value = conn
        .get("SIM_CONFIG")
        .with_context(|| "failed to read config")
        .and_then(|payload: Vec<u8>| {
            serde_json::from_slice(payload.as_slice())
                .with_context(|| "Config msg deserialization failure")
        })
        .expect("Failed to read payload");

    let resp = with_status(warp::reply::json(&conf), StatusCode::OK);
    Ok(resp)
}

pub async fn terrain_rooms(db: PgPool) -> Result<impl warp::Reply, Infallible> {
    struct RoomId {
        q: i32,
        r: i32,
    };

    let res = sqlx::query_as!(
        RoomId,
        "
        SELECT q, r
        FROM world_map;
        "
    )
    .fetch_all(&db)
    .await
    .expect("Failed to query world");

    let res = res
        .into_iter()
        .map(|RoomId { q, r }| AxialPoint { q, r })
        .collect::<Vec<_>>();

    let resp = warp::reply::json(&res);

    Ok(resp)
}

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Failed to compile script {0}")]
    CompileError(compiler::CompilationError),
    #[error("User info was not found. Did you log in?")]
    Unauthorized,
    #[error("Unexpected error while attempting to commit the script: {0}")]
    InternalError(anyhow::Error),
    #[error("Failed to send update script command: {0}")]
    CommandError(CommandError),
}

impl warp::reject::Reject for ScriptError {}

impl ScriptError {
    pub fn status(&self) -> StatusCode {
        match self {
            ScriptError::CommandError(ref err) => err.status(),
            ScriptError::CompileError(_) => StatusCode::BAD_REQUEST,
            ScriptError::Unauthorized => StatusCode::UNAUTHORIZED,
            ScriptError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn compile(
    logger: Logger,
    cu: CompilationUnit,
) -> Result<impl warp::Reply, warp::Rejection> {
    match compiler::compile(None, cu) {
        Ok(res) => {
            trace!(logger, "compilation succeeded {:?}", res);
            let resp = Box::new(StatusCode::NO_CONTENT);
            Ok(resp)
        }
        Err(err) => {
            debug!(logger, "compilation failed {}", err);
            Err(warp::reject::custom(ScriptError::CompileError(err)))
        }
    }
}

pub async fn get_script(
    id: Uuid,
    identity: Option<Identity>,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let identity = identity.ok_or_else(|| warp::reject::not_found())?;

    let user_id = identity.user_id;
    let script = sqlx::query_file_as!(
        ScriptEntity,
        "src/handler/get_users_script.sql",
        user_id,
        id
    )
    .fetch_optional(&db)
    .await
    .with_context(|| "Failed to query scripts")
    .map_err(ScriptError::InternalError)
    .map_err(warp::reject::custom)?;

    script
        .map(|script| warp::reply::json(&script))
        .ok_or_else(|| warp::reject::not_found())
}

pub async fn list_scripts(
    identity: Option<Identity>,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let identity = identity.ok_or_else(|| warp::reject::not_found())?;

    let user_id = identity.user_id;
    let scripts = sqlx::query_file_as!(
        ScriptMetadata,
        "src/handler/list_users_scripts.sql",
        user_id
    )
    .fetch_all(&db)
    .await
    .with_context(|| "Failed to query scripts")
    .map_err(ScriptError::InternalError)
    .map_err(warp::reject::custom)?;

    let res = warp::reply::json(&scripts);
    Ok(res)
}
