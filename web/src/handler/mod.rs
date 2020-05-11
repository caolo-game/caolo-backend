mod auth;

pub use auth::*;

use crate::model::User;
use crate::protos::schema::Schema;
use crate::RedisPool;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use protobuf::parse_from_bytes;
use redis::Commands;

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}

#[get("/myself")]
pub async fn myself(user: Option<User>) -> Result<HttpResponse, HttpResponse> {
    user.map(|user: User| HttpResponse::Ok().json(user))
        .ok_or_else(|| HttpResponse::NotFound().finish())
}

#[get("/schema")]
pub async fn schema(cache: web::Data<RedisPool>) -> Result<HttpResponse, error::Error> {
    let mut conn = cache
        .into_inner()
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    conn.get("SCHEMA")
        .map_err(actix_web::error::ErrorInternalServerError)
        .and_then(|schema: Vec<u8>| {
            parse_from_bytes(schema.as_slice()).map_err(actix_web::error::ErrorInternalServerError)
        })
        .map(|schema: Schema| HttpResponse::Ok().json(schema))
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res| HttpResponse::NoContent().finish())
        .map_err(error::ErrorBadRequest)
}
