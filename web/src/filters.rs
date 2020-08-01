//! Warp filters.
//!
//! Entry point filters will call handlers to execute logic.
//!
//!
use crate::config::*;
use crate::handler;
use crate::model;
use crate::world;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{o, trace, warn, Logger};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use warp::http::StatusCode;
use warp::reply::with_status;
use warp::Filter;

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let response = with_status(warp::reply(), StatusCode::NO_CONTENT);
    Ok(response)
}

pub fn api(
    logger: Logger,
    conf: Config,
    cache_pool: r2d2::Pool<RedisConnectionManager>,
    db_pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let conf = std::sync::Arc::new(conf);

    let cache_pool = {
        let filter = warp::any().map(move || cache_pool.clone());
        move || filter.clone()
    };

    let db_pool = {
        let filter = warp::any().map(move || db_pool.clone());
        move || filter.clone()
    };

    let config = {
        let filter = warp::any().map(move || {
            let conf = Arc::clone(&conf);
            conf
        });
        move || filter.clone()
    };

    let jwks_cache = {
        let cache = Arc::new(RwLock::new(std::mem::MaybeUninit::uninit()));
        let filter = warp::any().map(move || Arc::clone(&cache));
        move || filter.clone()
    };

    let jwks = {
        let logger = logger.clone();
        let filter = warp::any()
            .and(warp::any().map(move || logger.clone()))
            .and(jwks_cache())
            .and_then(model::load_jwks);
        move || filter.clone()
    };

    // I used `and + optional` instead of `or` because a lack of `authorization` is not inherently
    // and error, however `or` would return 400 if neither method is used
    let identity = {
        let logger = logger.clone();
        let identity = warp::any()
            .and(config())
            .and(warp::filters::header::optional::<String>("authorization"))
            .and(warp::filters::cookie::optional("authorization"))
            .and(jwks())
            .map(
                move |config: Arc<Config>,
                      header_id: Option<String>,
                      cookie_id: Option<String>,
                      jwks: &model::JWKS| {
                    header_id.or(cookie_id).and_then(|token: String| {
                        trace!(logger, "deseralizing Identity: {:?}", token);
                        let kid = alcoholic_jwt::token_kid(&token)
                            .expect("failed to find token")
                            .expect("token was empty");
                        let jwk = jwks.find(kid.as_str())?;
                        let validations = vec![alcoholic_jwt::Validation::Audience(
                            config.auth_token_audience.clone(),
                        )];
                        let token = alcoholic_jwt::validate(&token, jwk, validations)
                            .map_err(|e| {
                                warn!(logger, "token deserialization failed {:?}", e);
                            })
                            .ok()?;
                        let id: model::Identity = serde_json::from_value(token.claims)
                            .expect("failed to deserialize claims");
                        slog::debug!(logger, "WIN {:?}", id);
                        Some(id)
                    })
                },
            );
        move || identity.clone()
    };

    let current_user = {
        let current_user = warp::any()
            .and(identity())
            .and(db_pool())
            .and_then(model::current_user);
        move || current_user.clone()
    };

    let logger = {
        let filter = warp::any().and(warp::addr::remote()).and(identity()).map(
            move |addr: Option<std::net::SocketAddr>, id: Option<model::Identity>| {
                logger.new(o!(
                    "current_user" => id.map(|id|format!("{:?}", id.id)),
                    "address" => addr
                ))
            },
        );
        move || filter.clone()
    };

    let health_check = warp::get().and(warp::path("health")).and_then(health_check);
    let world_stream = warp::get()
        .and(warp::path("world"))
        .and(logger())
        .and(warp::ws())
        .and(current_user())
        .and(cache_pool())
        .map(move |logger: Logger, ws: warp::ws::Ws, user, pool| {
            ws.on_upgrade(move |socket| world::world_stream(logger, socket, user, pool))
        });

    let myself = warp::get()
        .and(warp::path("myself"))
        .and(current_user())
        .and_then(handler::myself);

    let schema = warp::get()
        .and(warp::path("schema"))
        .and(logger())
        .and(cache_pool())
        .and_then(handler::schema);

    let terrain_rooms = warp::get()
        .and(warp::path!("terrain" / "rooms"))
        .and(db_pool())
        .and_then(handler::terrain_rooms);

    let terrain = warp::get()
        .and(warp::path("terrain"))
        .and(logger())
        .and(warp::query())
        .and(db_pool())
        .and_then(handler::terrain);

    let compile = warp::post()
        .and(warp::path("compile"))
        .and(logger())
        .and(warp::filters::body::json())
        .and_then(handler::compile);

    let save_script = warp::post()
        .and(warp::path!("scripts" / "commit"))
        .and(logger())
        .and(current_user())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and(cache_pool())
        .and_then(handler::save_script);

    let register = warp::post()
        .and(warp::path!("user" / "register"))
        .and(logger())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and_then(handler::register);

    health_check
        .or(world_stream)
        .or(myself)
        .or(schema)
        .or(terrain_rooms)
        .or(terrain)
        .or(save_script)
        .or(compile)
        .or(register)
}
