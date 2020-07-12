// mod auth;
//
// pub use auth::*;

use crate::model::User;
use crate::PgPool;
use crate::RedisPool;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use caolo_messages::{AxialPoint, Schema};
use redis::Commands;
use serde::Deserialize;

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
            rmp_serde::from_read_ref(schema.as_slice()).expect("Schema msg deserialization failure")
        })
        .map(|schema: Schema| HttpResponse::Ok().json(schema))
}

#[get("/terrain/rooms")]
pub async fn terrain_rooms(db: web::Data<PgPool>) -> Result<HttpResponse, error::Error> {
    struct RoomId {
        q: i32,
        r: i32,
    };
    let db = db.into_inner();

    let res = sqlx::query_as!(
        RoomId,
        "
        SELECT q, r
        FROM world_map;
        "
    )
    .fetch_all(&*db)
    .await
    .expect("Failed to query world");

    let res = res
        .into_iter()
        .map(|RoomId { q, r }| AxialPoint { q, r })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(res))
}

#[derive(Debug, Deserialize)]
pub struct TerrainQuery {
    q: i32,
    r: i32,
}

#[get("/terrain")]
pub async fn terrain(
    query: web::Query<TerrainQuery>,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, error::Error> {
    let db = db.into_inner();
    let TerrainQuery { q, r } = query.0;

    struct Res {
        payload: serde_json::Value,
    }

    let res = sqlx::query_as!(
        Res,
        "
        SELECT payload
        FROM world_map
        WHERE q=$1 AND r=$2
        ",
        q,
        r
    )
    .fetch_one(&*db)
    .await
    .map(|r| r.payload)
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => error::ErrorNotFound("Room was not found in the database"),
        _ => {
            log::error!("Failed to query database {:?}", e);
            error::ErrorInternalServerError("Failed to query database")
        }
    })?;

    let res = HttpResponse::Ok().json(res);
    Ok(res)
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res| HttpResponse::NoContent().finish())
        .map_err(error::ErrorBadRequest)
}
