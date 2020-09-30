//! Warp filters.
//!
//! Entry point filters will call handlers to execute logic.
//!
//!
use crate::config::*;
use crate::handler;
use crate::model;
use crate::world_state::refresh_state_job;
use crate::SharedState;
use cao_messages::WorldState;
use r2d2_redis::{r2d2, RedisConnectionManager};
use serde_json::json;
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
) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    let world_state = {
        let tick = tokio::time::Duration::from_millis(500); // TODO: read from conf
        let state: SharedState = Arc::new(RwLock::new(WorldState {
            rooms: Default::default(),
            logs: Default::default(),
            script_history: Default::default(),
        }));
        let refresh = refresh_state_job(
            "WORLD_STATE",
            cache_pool.clone(),
            logger.clone(),
            Arc::clone(&state),
            tick,
        );
        tokio::spawn(refresh);
        let filter = warp::any().map(move || Arc::clone(&state));
        move || filter.clone()
    };

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
        .and_then(handler::compile);

    let get_script = warp::get()
        .and(warp::path!("scripts" / Uuid))
        .and(identity())
        .and(db_pool())
        .and_then(handler::get_script);

    let list_scripts = warp::get()
        .and(warp::path("scripts"))
        .and(identity())
        .and(db_pool())
        .and_then(handler::list_scripts);

    let commit = warp::post()
        .and(warp::path!("scripts" / "commit"))
        .and(logger())
        .and(identity())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and(cache_pool())
        .and_then(handler::commit);

    let set_default_script = warp::post()
        .and(warp::path!("scripts" / "default-script"))
        .and(logger())
        .and(current_user())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and(cache_pool())
        .and_then(handler::set_default_script);

    let register = warp::post()
        .and(warp::path!("user" / "register"))
        .and(logger())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and_then(handler::register);

    let put_user = warp::put()
        .and(warp::path!("user"))
        .and(logger())
        .and(warp::filters::body::json())
        .and(db_pool())
        .and_then(handler::put_user);

    let read_bots_by_room = warp::get()
        .and(warp::path("bots"))
        .and(logger())
        .and(warp::query())
        .and(world_state())
        .and_then(handler::get_bots);

    let read_bot_history = warp::get()
        .and(logger())
        .and(warp::path!("bot-history"/u32))
        .and(world_state())
        .and_then(handler::get_bot_history);

    let get_room_objects = warp::get()
        .and(warp::path("room-objects"))
        .and(logger())
        .and(warp::query())
        .and(world_state())
        .and_then(handler::get_room_objects);

    let get_sim_config = warp::get()
        .and(warp::path("sim-config"))
        .and(cache_pool())
        .and_then(handler::get_sim_config);

    let place_structure = warp::post()
        .and(warp::path!("commands" / "place-structure"))
        .and(logger())
        .and(current_user())
        .and(cache_pool())
        .and(warp::filters::body::json())
        .and_then(handler::place_structure);

    health_check
        .or(get_room_objects)
        .or(myself)
        .or(schema)
        .or(terrain_rooms)
        .or(terrain)
        .or(get_script)
        .or(list_scripts)
        .or(commit)
        .or(set_default_script)
        .or(compile)
        .or(register)
        .or(put_user)
        .or(read_bots_by_room)
        .or(get_sim_config)
        .or(place_structure)
        .or(read_bot_history)
        .recover(handle_rejections)
}

async fn handle_rejections(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let status;
    let payload: serde_json::Value;

    if let Some(err) = err.find::<handler::UserRegistrationError>() {
        status = err.status();
        payload = json!({ "message": format!("{}", err) });
    } else if let Some(err) = err.find::<handler::CommandError>() {
        status = err.status();
        payload = json!({ "message": format!("{}", err) });
    } else if let Some(err) = err.find::<handler::ScriptError>() {
        status = err.status();
        payload = json!({ "message": format!("{}", err) });
    } else if err.is_not_found() {
        status = warp::http::StatusCode::NOT_FOUND;
        payload = json!({ "message": format!("{:?}", err) });
    } else {
        // if we allow a rejection to escape our filters CORS will not work
        status = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        payload = json!({ "message": format!("{:?}", err) });
    }

    Ok(with_status(reply::json(&payload), status))
}
