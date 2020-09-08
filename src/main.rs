mod config;
mod filters;
mod handler;
mod model;

pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{info, o, warn, Drain};
use sqlx::postgres::PgPool;
use warp::http::Method;
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
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec![
                    "Authorization",
                    "Content-Type",
                    "User-Agent",
                    "Sec-Fetch-Mode",
                    "Referer",
                    "Origin",
                    "Access-Control-Request-Method",
                    "Access-Control-Request-Headers",
                ])
                .allow_methods(vec![
                    Method::GET,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::POST,
                ]),
        )
        .with(warp::trace::request())
        .with(log_wrapper);

    info!(logger, "initialization done. starting the service...");
    warp::serve(api).run((host, port)).await;

    Ok(())
}
