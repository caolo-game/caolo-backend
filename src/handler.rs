mod commands;
mod rooms;
mod user;

pub use commands::*;
pub use rooms::*;
pub use user::*;

use crate::model::script::{Card, OwnedCard, Schema};
use crate::model::world::AxialPoint;
use crate::model::{Identity, ScriptEntity, ScriptMetadata};
use crate::PgPool;
use crate::SharedState;
use anyhow::Context;
use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::compiler::{self, CompilationUnit};
use slog::{debug, error, trace, Logger};
use std::convert::Infallible;
use thiserror::Error;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn schema(_logger: Logger, pool: PgPool) -> Result<impl warp::Reply, Infallible> {
    struct Foo {
        payload: serde_json::Value,
    }

    let Foo { payload } = sqlx::query_as!(
        Foo,
        r#"
    SELECT payload
    FROM scripting_schema
    ORDER BY id DESC
    LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .expect("read schema from db");

    let cards: Vec<OwnedCard> =
        serde_json::from_value(payload).expect("Failed to deserialize schema");

    let cards = cards.iter().map(|c| c.as_card()).collect();

    let mut schema = Schema { cards };

    let basic_schema = get_instruction_descriptions();
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

pub async fn get_sim_config(state: SharedState) -> Result<impl warp::Reply, Infallible> {
    let state = state.0.enter().unwrap();
    let conf = state.0.get("gameConfig");
    let status = if conf.is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    };
    let resp = with_status(warp::reply::json(&conf), status);
    Ok(resp)
}

pub async fn terrain_rooms(state: SharedState) -> Result<impl warp::Reply, Infallible> {
    let state = state.0.enter().unwrap();
    let keys = state
        .0
        .get("terrain")
        .and_then(|t| t.as_object())
        .map(|t| t.keys());

    let res = match keys {
        Some(keys) => keys
            .into_iter()
            .filter_map(|k| {
                let mut segments = k.split(';');
                let q = segments.next()?.parse().ok()?;
                let r = segments.next()?.parse().ok()?;
                Some(AxialPoint { q, r })
            })
            .collect(),
        None => {
            vec![]
        }
    };

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
    match compiler::compile(None, cu, None) {
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
