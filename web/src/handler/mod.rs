mod auth;

pub use auth::*;

use crate::model::User;
use crate::PgPool;
use actix_identity::Identity;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use log::{debug, error};

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}

#[get("/myself")]
pub async fn myself(id: Identity, db: web::Data<PgPool>) -> Result<HttpResponse, HttpResponse> {
    let id = id.identity().ok_or_else(|| {
        debug!("myself called without an identity");
        HttpResponse::Unauthorized().finish()
    })?;
    let db = db.into_inner();
    sqlx::query_as!(
        User,
        "
        SELECT ua.id, ua.display_name, ua.email, ua.created, ua.updated
        FROM user_account AS ua
        INNER JOIN user_credential AS uc
        ON uc.token = $1 AND uc.user_id = ua.id
        ",
        id
    )
    .fetch_optional(&*db)
    .await
    .map_err(|e| {
        error!("Failed to query user {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?
    .map(|user: User| HttpResponse::Ok().json(user))
    .ok_or_else(|| HttpResponse::NotFound().finish())
}

#[get("/schema")]
pub async fn schema() -> impl Responder {
    HttpResponse::NotImplemented().body("Helllo boii")
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res| HttpResponse::NoContent().finish())
        .map_err(error::ErrorBadRequest)
}
