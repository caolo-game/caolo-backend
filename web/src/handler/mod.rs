use crate::model::User;
use crate::PgPool;
use crate::RedisPool;
use anyhow::Context;
use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::compiler::{self, CompilationUnit};
use caolo_messages::{AxialPoint, Function, Schema};
use redis::Commands;
use serde::Deserialize;
use serde_json::Value;
use slog::{debug, error, trace, Logger};
use std::convert::Infallible;
use thiserror::Error;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn myself(user: Option<User>) -> Result<impl warp::Reply, Infallible> {
    let resp = warp::reply::json(&user);
    let resp = match user {
        Some(_) => with_status(resp, StatusCode::OK),
        None => with_status(resp, StatusCode::NOT_FOUND),
    };
    Ok(resp)
}

#[derive(Deserialize, Debug)]
pub struct UserRegistrationData {
    pub id: String,
    pub tenant: String,
    pub username: String,
    pub email: String,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    pub user_metadata: Value,
}

#[derive(Debug, Error)]
pub enum UserRegistrationError {
    #[error("This user ID has already been registered")]
    UserIdTaken,
    #[error("This e-mail address has already been registered")]
    EmailTaken,
    #[error("Unexpected error while attempting to register new user")]
    InternalError,
}

impl warp::reject::Reject for UserRegistrationError {}

impl UserRegistrationError {
    pub fn status(&self) -> StatusCode {
        match self {
            UserRegistrationError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserRegistrationError::UserIdTaken | UserRegistrationError::EmailTaken => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

pub async fn register(
    logger: Logger,
    payload: UserRegistrationData,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = sqlx::query!(
        "
        INSERT INTO user_account
            (auth0_id, display_name, email, email_verified)
        VALUES
            ($1, $2, $3, $4)
        ",
        payload.id,
        payload.username,
        payload.email,
        payload.email_verified,
    )
    .execute(&db)
    .await;
    if let Err(err) = res {
        use sqlx::Error;
        match err {
            Error::Database(ref err) => match err.constraint_name() {
                Some(constraint) if constraint == "email_is_unique" => {
                    let err = UserRegistrationError::EmailTaken;
                    return Err(warp::reject::custom(err));
                }
                Some(constraint) if constraint == "auth0_id_is_unique" => {
                    let err = UserRegistrationError::UserIdTaken;
                    return Err(warp::reject::custom(err));
                }
                _ => {}
            },
            _ => {}
        }
        error!(logger, "Unexpected error while registering user {:?}", err);
        let err = UserRegistrationError::InternalError;
        return Err(warp::reject::custom(err));
    }
    let res = warp::reply::reply();
    Ok(res)
}

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
}

impl warp::reject::Reject for CompileError {}

impl CompileError {
    pub fn status(&self) -> StatusCode {
        match self {
            CompileError::CompileError(_) => StatusCode::BAD_REQUEST,
            CompileError::Unauthorized => StatusCode::UNAUTHORIZED,
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
    user: Option<User>,
    cu: CompilationUnit,
    _db: PgPool,
    _cache: RedisPool,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let _user = user.ok_or_else(|| warp::reject::custom(CompileError::Unauthorized))?;

    let _program = match compiler::compile(None, cu) {
        Ok(res) => res,
        Err(err) => {
            debug!(logger, "compilation failure {:?}", err);
            return Err(warp::reject::custom(CompileError::CompileError(err)));
        }
    };
    unimplemented!()
}
