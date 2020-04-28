use actix_web::web::HttpResponse;
use actix_web::{get, Responder};

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}
