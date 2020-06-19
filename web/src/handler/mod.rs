mod auth;

pub use auth::*;

use crate::model::User;
use caolo_messages::{Schema, WorldTerrain};
use crate::RedisPool;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
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
        .map(|schema: Vec<u8>| {
            rmp_serde::from_read_ref(schema.as_slice())
                .expect("Schema msg deserialization failure")
        })
        .map(|schema: Schema| HttpResponse::Ok().json(schema))
}

#[get("/terrain")]
pub async fn terrain(cache: web::Data<RedisPool>) -> Result<HttpResponse, error::Error> {
    let mut conn = cache
        .into_inner()
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    conn.get("WORLD_TERRAIN")
        .map_err(actix_web::error::ErrorInternalServerError)
        .map(|terrain: Vec<u8>| {
            rmp_serde::from_read_ref(terrain.as_slice()).expect("Terrain msg deserialization failure")
        })
        .map(|terrain: WorldTerrain| HttpResponse::Ok().json(terrain))
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res| HttpResponse::NoContent().finish())
        .map_err(error::ErrorBadRequest)
}
