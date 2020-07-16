mod auth;

use crate::model::User;
use crate::PgPool;
use crate::RedisPool;
use anyhow::Context;
pub use auth::*;
use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::compiler::{self, CompilationUnit};
use caolo_messages::{AxialPoint, Function, Schema};
use log::{debug, error, trace};
use redis::Commands;
use serde::Deserialize;
use std::convert::Infallible;
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

pub async fn schema(cache: RedisPool) -> Result<impl warp::Reply, Infallible> {
    let mut conn = cache.get().expect("failed to aquire cache connection");

    let basic_schema = get_instruction_descriptions();

    let schema: Result<Schema, _> = conn
        .get("SCHEMA")
        .map_err(|err| {
            error!("Failed to read schema {:?}", err);
            err
        })
        .with_context(|| "failed to read schema")
        .and_then(|schema: Vec<u8>| {
            rmp_serde::from_read_ref(schema.as_slice())
                .with_context(|| "Schema msg deserialization failure")
        });
    let resp = match schema {
        Ok(mut schema) => {
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
            with_status(warp::reply::json(&schema), StatusCode::OK)
        }
        Err(err) => {
            error!("Failed to read schema {:?}", err);
            with_status(
                warp::reply::json(&Option::<()>::None),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };
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

pub async fn terrain(query: TerrainQuery, db: PgPool) -> Result<impl warp::Reply, Infallible> {
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
            error!("Failed to query database {:?}", e);
            let resp = warp::reply::json(&Option::<()>::None);
            Ok::<_, Infallible>(with_status(resp, StatusCode::INTERNAL_SERVER_ERROR))
        }
    })
    .unwrap();

    Ok(res)
}

pub async fn compile(cu: CompilationUnit) -> Result<Box<dyn warp::Reply>, Infallible> {
    match compiler::compile(cu) {
        Ok(res) => {
            trace!("compilation succeeded {:?}", res);
            let resp = Box::new(StatusCode::NO_CONTENT);
            Ok(resp)
        }
        Err(err) => {
            debug!("compilation failed {:?}", err);
            let resp = warp::reply::json(&err);
            let resp = Box::new(with_status(resp, StatusCode::BAD_REQUEST));
            Ok(resp)
        }
    }
}
