use crate::model::User;
use crate::RedisPool;
use cao_messages::{AxialPoint, Bot, WorldState};
use redis::Commands;
use slog::{error, Logger};

pub async fn get_bots(
    logger: Logger,
    _user: Option<User>,
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
    Ok(state
        .bots
        .into_iter()
        .filter(|bot| bot.position.room == room)
        .collect())
}
