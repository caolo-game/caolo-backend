//! Room specific handlers
//!

use crate::model::world::AxialPoint;
use crate::SharedState;
use arrayvec::ArrayString;
use serde::Deserialize;
use slog::{warn, Logger};
use std::fmt::Write;
use std::{collections::HashMap, convert::Infallible};
use warp::http::StatusCode;

fn axial_to_string(p: AxialPoint) -> ArrayString<[u8; 16]> {
    let mut res = ArrayString::new();
    write!(res, "{};{}", p.q, p.r).expect("Failed to serialize axial");
    res
}

pub async fn terrain(
    _logger: Logger,
    room_pos: AxialPoint,
    state: SharedState,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let state = state.0.enter().unwrap();
    let res = state
        .payload
        .get("terrain")
        .and_then(|t| t.get(axial_to_string(room_pos).as_str()));
    let response: Box<dyn warp::Reply> = match res {
        Some(ref t) => Box::new(warp::reply::json(t)),
        None => Box::new(warp::reply::with_status(
            warp::reply(),
            StatusCode::NOT_FOUND,
        )),
    };
    Ok(response)
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
    _logger: Logger,
    RoomObjectsQuery {
        q,
        r,
        bots: projection_bots,
        resources: projection_resources,
        structures: projection_structures,
    }: RoomObjectsQuery,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.0.enter().unwrap();

    let room_id = axial_to_string(AxialPoint { q, r });
    let mut result = HashMap::new();

    macro_rules! project {
        ($projection: ident, $key: expr) => {
            if !$projection.map(|x| x == 0).unwrap_or(false) {
                // if projection.$key is not 0
                // set the key to the value in state
                result.insert(
                    $key,
                    state
                        .payload
                        .get($key)
                        .and_then(|t| t.get(room_id.as_str())),
                );
            }
        };
    }

    project!(projection_bots, "bots");
    project!(projection_structures, "structures");
    project!(projection_resources, "resources");

    let result = serde_json::json!({
        "time": state.time,
        "payload": result
    });

    let response = warp::reply::json(&result);
    Ok(response)
}

pub async fn get_bots(
    logger: Logger,
    room: AxialPoint,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.0.enter().unwrap();
    let list = state
        .payload
        .get("bots")
        .and_then(|bots| bots.get(axial_to_string(room).as_str()))
        .ok_or_else(|| {
            warn!(logger, "room does not exist {:?}", room);
            warp::reject()
        })?;
    let response = warp::reply::json(list);
    Ok(response)
}
