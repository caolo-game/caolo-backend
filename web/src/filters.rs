//! Warp filters.
//!
//! Entry point filters will call handlers to execute logic.
//!
//!
use crate::config::*;
use crate::handler;
use crate::model;
use crate::world;
use log::{trace, warn};
use r2d2_redis::{r2d2, RedisConnectionManager};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::reply::with_status;
use warp::Filter;

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let response = with_status(warp::reply(), StatusCode::NO_CONTENT);
    Ok(response)
}

pub fn api(
    conf: Config,
    cache_pool: r2d2::Pool<RedisConnectionManager>,
    db_pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply> + Clone {
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

    // I used `and + optional` instead of `or` because a lack of `authorization` is not inherently
    // and error, however `or` would return 400 if neither method is used
    let identity = warp::filters::header::optional::<String>("authorization")
        .and(warp::filters::cookie::optional("authorization"))
        .map(|header_id: Option<String>, cookie_id: Option<String>| {
            header_id.or(cookie_id).and_then(|cookie: String| {
                trace!("deseralizing Identity: {:?}", cookie);
                let id: model::Identity = FromStr::from_str(cookie.as_str())
                    .map_err(|e| {
                        warn!("identity cookie deserialization failed {:?}", e);
                    })
                    .ok()?;
                Some(id)
            })
        });

    let current_user = {
        let current_user = warp::any()
            .and(identity)
            .and(db_pool())
            .and_then(move |id, db_pool| model::current_user(id, db_pool));
        move || current_user.clone()
    };

    let extend_token = warp::get()
        .and(warp::path("extend-token"))
        .and(config())
        .and(identity)
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
        .and(warp::ws())
        .and(current_user())
        .and(cache_pool())
        .map(move |ws: warp::ws::Ws, user, pool| {
            ws.on_upgrade(move |socket| world::world_stream(socket, user, pool))
        });

    let myself = warp::get()
        .and(warp::path("myself"))
        .and(current_user())
        .and_then(handler::myself);

    let schema = warp::get()
        .and(warp::path("schema"))
        .and(cache_pool())
        .and_then(handler::schema);

    let terrain_rooms = warp::get()
        .and(warp::path!("terrain" / "rooms"))
        .and(db_pool())
        .and_then(handler::terrain_rooms);

    let terrain = warp::get()
        .and(warp::path("terrain"))
        .and(warp::query())
        .and(db_pool())
        .and_then(handler::terrain);

    let compile = warp::post()
        .and(warp::path("compile"))
        .and(warp::filters::body::json())
        .and_then(handler::compile);

    let google_login_redirect = warp::get()
        .and(warp::path!("login" / "google" / "redirect"))
        .and(warp::cookie("session_id"))
        .and(warp::query())
        .and(config())
        .and(cache_pool())
        .and(db_pool())
        .and_then(handler::login_redirect);

    let google_login = warp::get()
        .and(warp::path!("login" / "google"))
        .and(warp::query())
        .and(config())
        .and(cache_pool())
        .and_then(handler::login);

    let api = health_check
        .or(world_stream)
        .or(myself)
        .or(schema)
        .or(terrain_rooms)
        .or(terrain)
        .or(google_login_redirect)
        .or(google_login)
        .or(extend_token)
        .or(compile);

    api.with(warp::log("caolo_web-router"))
        .with(warp::cors().allow_any_origin().allow_credentials(true))
}
