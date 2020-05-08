use crate::model::User;
use crate::PgPool;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use log::error;
use sqlx;
use uuid::Uuid;

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}

#[get("/myself")]
pub async fn myself(pool: web::Data<PgPool>) -> Result<HttpResponse, HttpResponse> {
    let user_id = Uuid::default();
    sqlx::query_as!(
        User,
        "
        SELECT id, display_name, email, created, updated
        FROM user_account
        WHERE id = $1
        ",
        user_id
    )
    .fetch_optional(&**pool)
    .await
    .map_err(|e| {
        error!("Failed to query user {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?
    .map(|_user| unimplemented!())
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
