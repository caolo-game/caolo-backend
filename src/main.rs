mod config;
mod filters;
mod handler;
mod model;
mod world;

pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{info, o, warn, Drain};
use sqlx::postgres::PgPool;
use warp::reply::{self, with_status};
use warp::Filter;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "web-dotenv")]
    dotenv().ok();

    let logger = {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_envlogger::new(drain).fuse();
        let drain = slog_async::Async::new(drain)
            .overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
            .chan_size(4096)
            .build()
            .fuse();
        slog::Logger::root(drain, o!())
    };

    let log_wrapper = {
        let logger = logger.clone();
        warp::log::custom(move |info| {
            let t = info.elapsed().as_micros();

            let ms = t / 1000;
            let us = t % 1000;

            info!(
                logger,
                "[{:?}]: {} {} {} done in {}ms {}μs",
                info.remote_addr(),
                info.method(),
                info.path(),
                info.status(),
                ms,
                us
            );
        })
    };

    info!(logger, "initializing Sentry");
    let _sentry = std::env::var("SENTRY_URI")
        .ok()
        .map(|uri| {
            let options: sentry::ClientOptions = uri.as_str().into();
            sentry::init(options)
        })
        .ok_or_else(|| {
            warn!(logger, "Sentry URI was not provided");
        });

    info!(logger, "reading config");
    let conf = Config::read(logger.clone()).unwrap();

    info!(logger, "initializing Redis pool");
    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    info!(logger, "initializing Postgres pool");
    let db_pool = PgPool::builder()
        .max_size(8)
        .build(&conf.db_url)
        .await
        .unwrap();

    let host = conf.host;
    let port = conf.port;

    info!(logger, "initializing API");
    let api = filters::api(logger.clone(), conf, cache_pool, db_pool)
        .recover(handle_rejection)
        .with(log_wrapper)
        .with(warp::trace::request())
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_credentials(true)
                .allow_header("authorization")
                .allow_header("content-type")
                .allow_method(warp::http::Method::GET)
                .allow_method(warp::http::Method::PUT)
                .allow_method(warp::http::Method::DELETE)
                .allow_method(warp::http::Method::OPTIONS)
                .allow_method(warp::http::Method::POST),
        );

    info!(logger, "initialization done. starting the service...");
    warp::serve(api).run((host, port)).await;

    Ok(())
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    let status;
    let payload: serde_json::Value;
    if let Some(err) = err.find::<handler::ScriptError>() {
        status = err.status();
        payload = format!("{}", err).into();
    } else if let Some(err) = err.find::<handler::UserRegistrationError>() {
        status = err.status();
        payload = format!("{}", err).into();
    } else {
        return Err(err);
    }
    Ok(with_status(reply::json(&payload), status))
}
