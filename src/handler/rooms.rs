//! Room specific handlers
//!

use crate::PgPool;
use crate::RedisPool;
use cao_messages::{AxialPoint, Bot, Resource, Structure, WorldState};
use redis::Commands;
use slog::{error, Logger};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn terrain(
    logger: Logger,
    AxialPoint { q, r }: AxialPoint,
    db: PgPool,
) -> Result<impl warp::Reply, Infallible> {
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

pub async fn get_bots(
    logger: Logger,
    pool: RedisPool,
    room: AxialPoint,
) -> Result<impl warp::Reply, warp::Rejection> {
    let bots = read_bots(&logger, room, &pool).expect("Failed to read world");
    let response = warp::reply::json(&bots);
    Ok(response)
}

fn read_bots(logger: &Logger, room: AxialPoint, pool: &RedisPool) -> anyhow::Result<Vec<Bot>> {
    let mut connection = pool.get().unwrap();
    let state: WorldState = connection
        .get::<_, Vec<u8>>("WORLD_STATE")
        .map(|bytes| {
            rmp_serde::from_read_ref(bytes.as_slice()).expect("WorldState deserialization error")
        })
        .map_err(|err| {
            error!(logger, "Failed to read world state {:?}", err);
            err
        })?;
    let bots = state
        .bots
        .into_iter()
        .filter(|bot| bot.position.room == room)
        .collect();
    Ok(bots)
}

pub async fn get_structures(
    logger: Logger,
    pool: RedisPool,
    room: AxialPoint,
) -> Result<impl warp::Reply, warp::Rejection> {
    let structures = read_structures(&logger, room, &pool).expect("Failed to read world");
    let response = warp::reply::json(&structures);
    Ok(response)
}

fn read_structures(
    logger: &Logger,
    room: AxialPoint,
    pool: &RedisPool,
) -> anyhow::Result<Vec<Structure>> {
    let mut connection = pool.get().unwrap();
    let state: WorldState = connection
        .get::<_, Vec<u8>>("WORLD_STATE")
        .map(|bytes| {
            rmp_serde::from_read_ref(bytes.as_slice()).expect("WorldState deserialization error")
        })
        .map_err(|err| {
            error!(logger, "Failed to read world state {:?}", err);
            err
        })?;
    let structures = state
        .structures
        .into_iter()
        .filter(|bot| bot.position.room == room)
        .collect();
    Ok(structures)
}

pub async fn get_resources(
    logger: Logger,
    pool: RedisPool,
    room: AxialPoint,
) -> Result<impl warp::Reply, warp::Rejection> {
    let resources = read_resources(&logger, room, &pool).expect("Failed to read world");
    let response = warp::reply::json(&resources);
    Ok(response)
}

fn read_resources(
    logger: &Logger,
    room: AxialPoint,
    pool: &RedisPool,
) -> anyhow::Result<Vec<Resource>> {
    let mut connection = pool.get().unwrap();
    let state: WorldState = connection
        .get::<_, Vec<u8>>("WORLD_STATE")
        .map(|bytes| {
            rmp_serde::from_read_ref(bytes.as_slice()).expect("WorldState deserialization error")
        })
        .map_err(|err| {
            error!(logger, "Failed to read world state {:?}", err);
            err
        })?;
    let resources = state
        .resources
        .into_iter()
        .filter(|bot| bot.position.room == room)
        .collect();
    Ok(resources)
}
