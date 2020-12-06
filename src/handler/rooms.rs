//! Room specific handlers
//!

use crate::model::world::{AxialPoint, WorldState};
use crate::SharedState;
use serde::Deserialize;
use slog::{warn, Logger};
use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, RwLock},
};
use warp::http::StatusCode;

pub async fn terrain(
    _logger: Logger,
    AxialPoint { q, r }: AxialPoint,
    state: Arc<RwLock<WorldState>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let state = state.read().unwrap();
    let res = state
        .0
        .get("terrain")
        .and_then(|t| t.get(&format!("{};{}", q, r)));
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
    let state = state.read().unwrap();

    let room_id = format!("{};{}", q, r);
    let mut result = HashMap::new();

    macro_rules! project {
        ($projection: ident, $key: expr) => {
            if !$projection.map(|x| x == 0).unwrap_or(false) {
                // if projection.$key is not 0
                // set the key to the value in state
                result.insert($key, state.0.get($key).and_then(|t| t.get(&room_id)));
            }
        };
    };

    project!(projection_bots, "bots");
    project!(projection_structures, "structures");
    project!(projection_resources, "resources");

    let response = warp::reply::json(&result);

    Ok(response)
}

pub async fn get_bots(
    logger: Logger,
    room: AxialPoint,
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state = state.read().unwrap();
    let list = state
        .0
        .get("bots")
        .and_then(|bots| bots.get(&format!("{};{}", room.q, room.r)))
        .ok_or_else(|| {
            warn!(logger, "room does not exist {:?}", room);
            warp::reject()
        })?;
    let response = warp::reply::json(list);
    Ok(response)
}
