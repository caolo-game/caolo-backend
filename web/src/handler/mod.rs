mod auth;

pub use auth::*;

use crate::model::User;
use actix_web::web::{HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};

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
pub async fn schema() -> impl Responder {
    HttpResponse::NotImplemented().body("Helllo boii")
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res| HttpResponse::NoContent().finish())
        .map_err(error::ErrorBadRequest)
}
