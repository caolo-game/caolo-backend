// for protobuf
#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde_derive;

mod handler;
mod model;
mod protos;
mod world;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware, App, HttpServer};
use r2d2_redis::{r2d2, RedisConnectionManager};
use sqlx::postgres::PgPool;
use std::env;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub redis_url: String,
    pub db_url: String,
}

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

    let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_owned());

    let bind = format!("{}:{}", host, port);

    let conf = Config {
        allowed_origins: vec!["http://localhost:3000".to_owned()],
        redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/0".to_owned()),
        db_url: env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/caolo".to_owned()),
    };

    let cache_manager = RedisConnectionManager::new(conf.redis_url.as_str()).unwrap();
    let cache_pool: RedisPool = r2d2::Pool::builder().build(cache_manager).unwrap();

    let db_pool = PgPool::builder().max_size(8).build(&conf.db_url).await.unwrap();

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
            .service(world::world_stream)
    })
    .bind(&bind)?
    .run()
    .await
}
