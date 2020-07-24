mod auth;
mod config;
mod filters;
mod google_auth;
mod handler;
mod model;
mod world;

pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{o, Drain};
use sqlx::postgres::PgPool;
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
        .with(warp::log("caolo_web-router"))
        .with(warp::cors().allow_any_origin().allow_credentials(true))
        .with(warp::trace::request());

    warp::serve(api).run((host, port)).await;

    Ok(())
}

async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if let Some(err) = err.find() {
        return Ok(handler::handle_compile_err(err));
    }
    unimplemented!()
}
