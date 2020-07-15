mod auth;
mod config;
mod google_auth;
mod handler;
mod model;
mod world;

pub use config::*;
use log::{trace, warn};
use r2d2_redis::{r2d2, RedisConnectionManager};
use sqlx::postgres::PgPool;
use std::convert::Infallible;
use std::str::FromStr;
use warp::Filter;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;

async fn health_check() -> Result<impl warp::Reply, Infallible> {
    let response = warp::http::Response::builder().body("healthy boi").unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "web-dotenv")]
    dotenv().ok();

    let _guard = std::env::var("SENTRY_URI")
        .ok()
        .map(|uri| sentry::init(uri));

    pretty_env_logger::init();

    let conf = Config::read().unwrap();

    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    let cache_pool = {
        let filter = warp::any().map(move || cache_pool.clone());
        move || filter.clone()
    };

    let db_pool = PgPool::builder()
        .max_size(8)
        .build(&conf.db_url)
        .await
        .unwrap();
    // PgPool has an Arc inside it so cloning should work as expected
    let db_pool = {
        let filter = warp::any().map(move || db_pool.clone());
        move || filter.clone()
    };

    let host = conf.host;
    let port = conf.port;

    let conf = std::sync::Arc::new(conf);
    let config = {
        let filter = warp::any().map(move || {
            let conf = std::sync::Arc::clone(&conf);
            conf
        });
        move || filter.clone()
    };

    let identity = warp::filters::header::optional::<String>("authorization")
        .and(warp::filters::cookie::optional("authorization"))
        .map(|header_id: Option<String>, cookie_id: Option<String>| {
            header_id.or(cookie_id).and_then(|cookie: String| {
                trace!("deseralizing Identity: {:?}", cookie);
                let id: model::Identity = FromStr::from_str(cookie.as_str())
                    .map_err(|e| {
                        // TODO: on expiration use the refresh token and issue a new token
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
        .and(identity)
        .and(current_user())
        .and_then(|id: Option<model::Identity>, user: Option<model::User>| {
            async move {
                match id.and_then(|id| user.map(|u| (id, u))) {
                    Some((id, _user)) => {
                        let new_id = model::Identity {
                            exp: (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp(),
                            ..id
                        };
                        Ok::<_, Infallible>(handler::set_identity(warp::reply(), new_id))
                    }
                    // the user is not logged in (or the token expired)
                    None => unimplemented!(),
                }
            }
        });

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

    warp::serve(
        api.with(warp::log("caolo_web-router"))
            .with(warp::cors().allow_any_origin().allow_credentials(true)),
    )
    .run((host, port))
    .await;

    Ok(())
}
