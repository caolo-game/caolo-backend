mod user;
pub use user::*;

use crate::model::Identity;
use crate::PgPool;
use crate::RedisPool;
use anyhow::Context;
use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::compiler::{self, CompilationUnit};
use cao_messages::{AxialPoint, Function, Schema};
use cao_messages::{CompiledScript, Label, UpdateScript};
use redis::Commands;
use serde::Deserialize;
use serde::Serialize;
use slog::{debug, error, trace, Logger};
use std::convert::Infallible;
use thiserror::Error;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn schema(_logger: Logger, cache: RedisPool) -> Result<impl warp::Reply, Infallible> {
    let mut conn = cache.get().expect("failed to aquire cache connection");

    let basic_schema = get_instruction_descriptions();

    let mut schema: Schema = conn
        .get("SCHEMA")
        .with_context(|| "failed to read schema")
        .and_then(|schema: Vec<u8>| {
            rmp_serde::from_read_ref(schema.as_slice())
                .with_context(|| "Schema msg deserialization failure")
        })
        .expect("Failed to read schema");

    schema
        .functions
        .extend(basic_schema.into_iter().map(|item| {
            Function::from_str_parts(
                item.name,
                item.description,
                item.input.as_ref(),
                item.output.as_ref(),
                item.params.as_ref(),
            )
        }));
    let resp = with_status(warp::reply::json(&schema), StatusCode::OK);
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

#[derive(Debug, Deserialize)]
pub struct TerrainQuery {
    q: i32,
    r: i32,
}

pub async fn terrain(
    logger: Logger,
    query: TerrainQuery,
    db: PgPool,
) -> Result<impl warp::Reply, Infallible> {
    let TerrainQuery { q, r } = query;

    struct Res {
        payload: serde_json::Value,
    }

    let res = sqlx::query_as!(
        Res,
        "
        SELECT payload
        FROM world_map
        WHERE q=$1 AND r=$2
        ",
        q,
        r
    )
    .fetch_one(&db)
    .await
    .map(|r| warp::reply::json(&r.payload))
    .map(|r| with_status(r, StatusCode::OK))
    .or_else(|e| match e {
        sqlx::Error::RowNotFound => {
            let resp = warp::reply::json(&Option::<()>::None);
            Ok(with_status(resp, StatusCode::NOT_FOUND))
        }
        _ => {
            error!(logger, "Failed to query database {:?}", e);
            let resp = warp::reply::json(&Option::<()>::None);
            Ok::<_, Infallible>(with_status(resp, StatusCode::INTERNAL_SERVER_ERROR))
        }
    })
    .unwrap();

    Ok(res)
}
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Failed to compile script {0}")]
    CompileError(compiler::CompilationError),
    #[error("User info was not found. Did you log in?")]
    Unauthorized,
    #[error("Unexpected error while attempting to commit the script: {0}")]
    InternalError(anyhow::Error),
}

impl warp::reject::Reject for CompileError {}

impl CompileError {
    pub fn status(&self) -> StatusCode {
        match self {
            CompileError::CompileError(_) => StatusCode::BAD_REQUEST,
            CompileError::Unauthorized => StatusCode::UNAUTHORIZED,
            CompileError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            Err(warp::reject::custom(CompileError::CompileError(err)))
        }
    }
}

pub async fn save_script(
    logger: Logger,
    identity: Option<Identity>,
    cu: CompilationUnit,
    db: PgPool,
    cache: RedisPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut tx = db.begin().await.expect("failed to begin transaction");

    let identity = identity.ok_or_else(|| warp::reject::custom(CompileError::Unauthorized))?;

    struct QueryRes {
        /// script_id
        id: Uuid,
        owner_id: Uuid,
    };

    let query = sqlx::query_as!(
        QueryRes,
        r#"
        INSERT INTO user_script
        (program, owner_id)
        VALUES
        (
            $1
            , (
                SELECT id AS owner_id
                FROM user_account
                WHERE auth0_id=$2
                LIMIT 1
            )
        )
        RETURNING id, owner_id;
        "#,
        serde_json::to_value(&cu).expect("failed to serialize CompilationUnit"),
        identity.user_id,
    )
    .fetch_one(&mut tx);

    let program = compiler::compile(None, cu).map_err(|err| {
        debug!(logger, "compilation failure {:?}", err);
        warp::reject::custom(CompileError::CompileError(err))
    })?;

    // map cao_lang script to cao_messages script
    let compiled_script = CompiledScript {
        bytecode: program.bytecode,
        labels: program
            .labels
            .into_iter()
            .map(|(k, cao_lang::Label { block, myself })| (k, Label { block, myself }))
            .collect(),
    };

    let QueryRes {
        id: script_id,
        owner_id: user_id,
    } = query
        .await
        .with_context(|| "failed to insert the program")
        .map_err(CompileError::InternalError)
        .map_err(warp::reject::custom)?;

    let msg = UpdateScript {
        script_id,
        user_id,
        compiled_script,
    };

    let mut conn = cache.get().expect("failed to get cache conn");
    let _: () = conn
        .lpush(
            "UPDATE_SCRIPT",
            serde_json::to_vec(&msg).expect("failed to serialize msg"),
        )
        .with_context(|| "Failed to send msg")
        .map_err(CompileError::InternalError)
        .map_err(warp::reject::custom)?;

    tx.commit().await.expect("failed to commit tx");

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SaveResult {
        script_id: Uuid,
    }

    let result = SaveResult { script_id };

    let result = warp::reply::json(&result);
    Ok(result)
}
