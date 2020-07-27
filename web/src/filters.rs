//! Warp filters.
//!
//! Entry point filters will call handlers to execute logic.
//!
//!
use crate::config::*;
use crate::handler;
use crate::model;
use crate::world;
use arrayvec::ArrayString;
use r2d2_redis::{r2d2, RedisConnectionManager};
use serde::Deserialize;
use slog::{debug, info, o, trace, warn, Logger};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::{Arc, Once, RwLock};
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
        let cache = Arc::new(RwLock::new(Vec::new()));
        let filter = warp::any().map(move || Arc::clone(&cache));
        move || filter.clone()
    };

    let jwks = {
        let logger = logger.clone();
        let filter = warp::any()
            .and(warp::any().map(move || logger.clone()))
            .and(jwks_cache())
            .and_then(load_jwks);
        move || filter.clone()
    };

    // I used `and + optional` instead of `or` because a lack of `authorization` is not inherently
    // and error, however `or` would return 400 if neither method is used
    let identity = {
        let logger = logger.clone();
        let identity = warp::any()
            .and(warp::filters::header::optional::<String>("authorization"))
            .and(warp::filters::cookie::optional("authorization"))
            .and(jwks())
            .map(
                move |header_id: Option<String>, cookie_id: Option<String>, jwks: &[JWK]| {
                    header_id.or(cookie_id).and_then(|token: String| {
                        trace!(
                            logger,
                            "deseralizing Identity: {:?} using jwks: {:?}",
                            token,
                            jwks
                        );
                        let id: model::Identity = FromStr::from_str(token.as_str())
                            .map_err(|e| {
                                warn!(logger, "identity token deserialization failed {:?}", e);
                            })
                            .ok()?;
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
                    "current_user" => id.map(|id|format!("{:?}", id.user_id)),
                    "address" => addr
                ))
            },
        );
        move || filter.clone()
    };

    let extend_token = warp::get()
        .and(warp::path("extend-token"))
        .and(config())
        .and(identity())
        .and(current_user())
        .and_then(
            |conf: Arc<Config>, id: Option<model::Identity>, user: Option<model::User>| async move {
                id.and_then(|id| user.map(|u| (id, u)))
                    .map(|(id, _user)| {
                        let new_id = model::Identity {
                            exp: (chrono::Utc::now() + conf.auth_token_duration).timestamp(),
                            ..id
                        };
                        let response = with_status(warp::reply(), StatusCode::NO_CONTENT);
                        handler::set_identity(response, new_id)
                    })
                    .ok_or_else(|| warp::reject::not_found())
            },
        );

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

    let google_login_redirect = warp::get()
        .and(warp::path!("login" / "google" / "redirect"))
        .and(logger())
        .and(warp::cookie("session_id"))
        .and(warp::query())
        .and(config())
        .and(cache_pool())
        .and(db_pool())
        .and_then(handler::login_redirect);

    let google_login = warp::get()
        .and(warp::path!("login" / "google"))
        .and(logger())
        .and(warp::query())
        .and(config())
        .and(cache_pool())
        .and_then(handler::login);

    health_check
        .or(world_stream)
        .or(myself)
        .or(schema)
        .or(terrain_rooms)
        .or(terrain)
        .or(google_login_redirect)
        .or(google_login)
        .or(save_script)
        .or(compile)
        .or(extend_token)
}

#[derive(Deserialize, Debug)]
pub struct JWK {
    pub x5c: Vec<String>,
    pub alg: ArrayString<[u8; 16]>,
    pub kid: ArrayString<[u8; 32]>,
}

#[derive(Deserialize, Debug)]
pub struct JWKS {
    pub keys: Vec<JWK>,
}

static JWKS_LOAD: Once = Once::new();

async fn load_jwks<'a>(
    logger: Logger,
    cache: Arc<RwLock<Vec<JWK>>>,
) -> Result<&'a [JWK], Infallible> {
    {
        let cache = Arc::clone(&cache);
        tokio::task::spawn_blocking(move || {
            JWKS_LOAD.call_once(|| {
                info!(logger, "performing initial JWK load");
                let cc = Arc::clone(&cache);
                let cache = cc;
                let uri = std::env::var("JWKS_URI")
                    .expect("Can not perform authorization without JWKS_URI");
                let payload = reqwest::blocking::get(&uri);
                let payload = payload.unwrap();
                let payload: JWKS = payload.json().unwrap();

                let mut cache = cache.write().unwrap();
                *cache.as_mut() = payload.keys;
                info!(logger, "JWK load finished");
                debug!(logger, "JWKs loaded: {:#?}", *cache);
            });
        })
        .await
        .expect("Failed to load JWKS");
    }

    let cache = cache.read().unwrap();
    let ptr = cache.as_ptr();
    unsafe { Ok(std::slice::from_raw_parts(ptr, cache.len())) }
}
