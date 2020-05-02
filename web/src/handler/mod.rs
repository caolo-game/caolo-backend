use actix_web::web::{HttpResponse, Json};
use actix_web::{post,get, Responder,error};
use cao_lang::compiler::{self, CompilationUnit};

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}

#[get("/myself")]
pub async fn myself() -> impl Responder {
    HttpResponse::NotImplemented().body("Helllo boii")
}

#[get("/schema")]
pub async fn schema() -> impl Responder {
    HttpResponse::NotImplemented().body("Helllo boii")
}

#[post("/compile")]
pub async fn compile(cu: Json<CompilationUnit>) -> impl Responder {
    compiler::compile(cu.into_inner())
        .map(|_res|{
            HttpResponse::NoContent()
                .finish()
        })
        .map_err(error::ErrorBadRequest)
}
