//! Room specific handlers
//!

use crate::model::world::AxialPoint;
use crate::PgPool;
use crate::SharedState;
use serde::Deserialize;
use slog::{error, warn, Logger};
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

#[derive(Deserialize)]
pub struct RoomObjectsQuery {
    pub q: i32,
    pub r: i32,

    // projections
    pub bots: Option<i32>,
    pub resources: Option<i32>,
    pub structures: Option<i32>,
}

/// ## Projection:
/// You can disable sending of certain fields by using the `<field-name>=0` query parameter
pub async fn get_room_objects(
    logger: Logger,
    RoomObjectsQuery {
        q,
        r,
        bots: projection_bots,
        resources: projection_resources,
        structures: projection_structures,
    }: RoomObjectsQuery,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let room = AxialPoint { q, r };
    let state = state.read().unwrap();
    let room = state.rooms.get(&room).ok_or_else(|| {
        warn!(logger, "Room {:?} does not exist", room);
        warp::reject()
    })?;
    let mut room = serde_json::to_value(room).expect("Failed to serialize room");

    macro_rules! deproject {
        ($projection: ident, $key: expr) => {
            if $projection.map(|x| x == 0).unwrap_or(false) {
                // if projection.$key is 0
                room[$key].take();
            }
        };
    };

    deproject!(projection_bots, "bots");
    deproject!(projection_structures, "structures");
    deproject!(projection_resources, "resources");

    let response = warp::reply::json(&room);

    Ok(response)
}

pub async fn get_bots(
    logger: Logger,
    room: AxialPoint,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().unwrap();
    let room = state.rooms.get(&room).ok_or_else(|| {
        warn!(logger, "room does not exist {:?}", room);
        warp::reject()
    })?;
    let response = warp::reply::json(&room.bots);
    Ok(response)
}
