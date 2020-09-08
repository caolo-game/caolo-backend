//! Warp filters.
//!
//! Entry point filters will call handlers to execute logic.
//!
//!
use crate::config::*;
use crate::handler;
use crate::model;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{o, trace, Logger};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use warp::http::StatusCode;
use warp::reply::{self, with_status};
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
            .and(warp::filters::header::optional("Authorization"))
            .and(jwks())
            .map(
                move |config: Arc<Config>, header_id: Option<String>, jwks: &model::JWKS| {
                    trace!(
                        logger,
                        "Deserializing identity from:\nheader\n{:?}",
                        header_id,
                    );
                    header_id
                        .as_ref()
                        .and_then(|id| {
                            const BEARER_PREFIX: &str = "Bearer ";
                            if !id.starts_with(BEARER_PREFIX) {
                                return None;
                            }
                            Some(&id[BEARER_PREFIX.len()..])
                        })
                        .and_then(|token: &str| {
                            trace!(logger, "Deserializing token {}", token);
                            model::Identity::validated_id(&logger, config.as_ref(), token, jwks)
                        })
                },
            );
        move || identity.clone()
    };

    let root_logger = {
        let filter =
            warp::any()
                .and(warp::addr::remote())
                .map(move |addr: Option<std::net::SocketAddr>| {
                    logger.new(o!(
                        "address" => addr
                    ))
                });
        move || filter.clone()
    };

    let current_user = {
        let current_user = warp::any()
            .and(root_logger())
            .and(identity())
            .and(db_pool())
            .and_then(model::current_user);
        move || current_user.clone()
    };

    let logger = {
        let filter = warp::any().and(root_logger()).and(identity()).map(
            move |logger: Logger, id: Option<model::Identity>| {
                logger.new(o!(
                    "user_id" => id.map(|id|format!("{:?}", id.user_id))
                ))
            },
        );
        move || filter.clone()
    };

    let health_check = warp::get().and(warp::path("health")).and_then(health_check);

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
        .and(warp::path!("scripts" / "compile"))
        .and(logger())
        .and(warp::filters::body::json())
        .and_then(handler::compile)
        .recover(handle_script_rejection);

    let get_script = warp::get()
        .and(warp::path!("scripts" / Uuid))
        .and(identity())
        .and(db_pool())
        .and_then(handler::get_script)
        .recover(handle_script_rejection);

    let list_scripts = warp::get()
        .and(warp::path("scripts"))
        .and(identity())
        .and(db_pool())
        .and_then(handler::list_scripts)
        .recover(handle_script_rejection);

    let commit = warp::post()
        .and(warp::path!("scripts" / "commit"))
        .and(logger())
        .and(identity())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and(cache_pool())
        .and_then(handler::commit)
        .recover(handle_script_rejection);

    let register = warp::post()
        .and(warp::path!("user" / "register"))
        .and(logger())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and_then(handler::register)
        .recover(handle_user_rejection);

    let put_user = warp::put()
        .and(warp::path!("user"))
        .and(logger())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and_then(handler::put_user)
        .recover(handle_user_rejection);

    let read_bots_by_room = warp::get()
        .and(warp::path("bots"))
        .and(logger())
        .and(cache_pool())
        .and(warp::query())
        .and_then(handler::get_bots);

    let read_structures_by_room = warp::get()
        .and(warp::path("structures"))
        .and(logger())
        .and(cache_pool())
        .and(warp::query())
        .and_then(handler::get_structures);

    let read_resources_by_room = warp::get()
        .and(warp::path("resources"))
        .and(logger())
        .and(cache_pool())
        .and(warp::query())
        .and_then(handler::get_resources);

    health_check
        .or(myself)
        .or(schema)
        .or(terrain_rooms)
        .or(terrain)
        .or(get_script)
        .or(list_scripts)
        .or(commit)
        .or(compile)
        .or(register)
        .or(put_user)
        .or(read_bots_by_room)
        .or(read_structures_by_room)
        .or(read_resources_by_room)
}

async fn handle_user_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(err) = err.find::<handler::UserRegistrationError>() {
        let status = err.status();
        let payload: serde_json::Value = format!("{}", err).into();
        Ok(with_status(reply::json(&payload), status))
    } else {
        Err(err)
    }
}

async fn handle_script_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(err) = err.find::<handler::ScriptError>() {
        let status = err.status();
        let payload: serde_json::Value = format!("{}", err).into();
        Ok(with_status(reply::json(&payload), status))
    } else {
        Err(err)
    }
}
