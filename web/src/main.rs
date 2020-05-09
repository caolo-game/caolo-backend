// for protobuf
#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde_derive;

mod config;
mod google_auth;
mod handler;
mod model;
mod protos;
mod world;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware, App, HttpServer};
pub use config::*;
use r2d2_redis::{r2d2, RedisConnectionManager};
use sqlx::postgres::PgPool;
use std::env;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

pub type RedisPool = r2d2::Pool<RedisConnectionManager>;

fn cors_options<'a>(allowed_origins: impl Iterator<Item = &'a str> + 'a) -> Cors {
    allowed_origins
        .fold(Cors::new(), |cors, o| {
            cors.allowed_origin(o).supports_credentials()
        })
        .allowed_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"])
        .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE])
        .max_age(3600)
        .supports_credentials()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    #[cfg(feature = "web-dotenv")]
    dotenv().ok();

    env_logger::init();

    let conf = Config::read().unwrap();

    let bind = format!("{}:{}", conf.host, conf.port);

    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    let db_pool = PgPool::builder()
        .max_size(8)
        .build(&conf.db_url)
        .await
        .unwrap();

    HttpServer::new(move || {
        let conf = conf.clone();
        let cache_pool = cache_pool.clone();
        let db_pool = db_pool.clone();
        let cors = cors_options(conf.allowed_origins.iter().map(|s| s.as_str()));
        App::new()
            .data(conf)
            .data(cache_pool)
            .data(db_pool)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[123; 32])
                    .name("authorization")
                    .secure(true),
            ))
            .wrap(cors.finish())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::new(
                r#"
    Remote IP: %a
    Started processing: %t
    First line: "%r"
    Status: %s
    Size: %b B
    Referer: "%{Referer}i"
    User-Agent: "%{User-Agent}i"
    Done in %D ms"#,
            ))
            .service(handler::index_page)
            .service(handler::myself)
            .service(handler::schema)
            .service(handler::compile)
            .service(handler::login)
            .service(handler::login_redirect)
            .service(world::world_stream)
    })
    .bind(&bind)?
    .run()
    .await
}
