mod handler;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware;
use actix_web::{http, App, HttpServer};
use std::env;
use std::sync::Arc;

#[cfg(feature = "web-dotenv")]
use dotenv::dotenv;

struct Config {
    allowed_origins: Vec<String>,
}

fn cors_options(config: &Config) -> Cors {
    config
        .allowed_origins
        .iter()
        .map(|string| string.as_str())
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

    let conf = Arc::new(Config {
        allowed_origins: vec!["localhost:3000".to_owned()],
    });

    HttpServer::new(move || {
        let conf = conf.clone();
        let cors = cors_options(&conf);
        App::new()
            .data(conf)
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
    })
    .bind(&bind)?
    .run()
    .await
}
