mod auth;
mod config;
mod filters;
mod google_auth;
mod handler;
mod model;
mod world;

pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{info, o, Drain};
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

    let _sentry = std::env::var("SENTRY_URI")
        .ok()
        .map(|uri| {
            let options: sentry::ClientOptions = uri.as_str().into();
            sentry::init(options)
        })
        .ok_or_else(|| {
            eprintln!("Sentry URI was not provided");
        });

    let conf = Config::read(logger.clone()).unwrap();

    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    let db_pool = PgPool::builder()
        .max_size(8)
        .build(&conf.db_url)
        .await
        .unwrap();

    let host = conf.host;
    let port = conf.port;

    let api = filters::api(logger, conf, cache_pool, db_pool)
        .recover(handle_rejection)
        .with(warp::cors().allow_any_origin().allow_credentials(true))
        .with(warp::trace::request())
        .with(log_wrapper);

    warp::serve(api).run((host, port)).await;

    Ok(())
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    let status;
    let payload: serde_json::Value;
    if let Some(err) = err.find::<handler::CompileError>() {
        status = err.status();
        payload = format!("{}", err).into();
    } else {
        return Err(err);
    }
    Ok(with_status(reply::json(&payload), status))
}
