use actix_web::web::HttpResponse;
use actix_web::{get, Responder};

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
