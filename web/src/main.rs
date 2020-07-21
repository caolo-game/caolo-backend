mod auth;
mod config;
mod filters;
mod google_auth;
mod handler;
mod model;
mod world;

pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use slog::{o, Drain, Logger};
use sqlx::postgres::PgPool;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "web-dotenv")]
    dotenv().ok();

    // TODO: async log
    let logger = Logger::root(slog_stdlog::StdLog.fuse(), o!());

    let _sentry = std::env::var("SENTRY_URI")
        .ok()
        .map(|uri| {
            let mut log_builder = pretty_env_logger::formatted_builder();
            let filters = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
            log_builder.parse_filters(filters.as_str());
            let log_integration = sentry_log::LogIntegration::default()
                .with_env_logger_dest(Some(log_builder.build()));
            let options: sentry::ClientOptions = uri.as_str().into();
            sentry::init(options.add_integration(log_integration))
        })
        .ok_or_else(|| {
            eprintln!("Sentry URI was not provided");
            pretty_env_logger::init();
        });

    let conf = Config::read().unwrap();

    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    let db_pool = PgPool::builder()
        .max_size(8)
        .build(&conf.db_url)
        .await
        .unwrap();

    let host = conf.host;
    let port = conf.port;

    let api = filters::api(logger, conf, cache_pool, db_pool);

    warp::serve(api).run((host, port)).await;

    Ok(())
}
