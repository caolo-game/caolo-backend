use crate::google_auth::oauth_client;
use crate::model::User;
use crate::Config;
use crate::PgPool;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use log::error;
use oauth2::{prelude::*, AuthorizationCode, CsrfToken};
use serde::Deserialize;
use sqlx::postgres::PgQueryAs;
use uuid::Uuid;

#[get("/")]
pub async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Helllo Worlllld")
}

#[derive(Deserialize, Debug)]
// TODO: remove debug
pub struct LoginQuery {
    pub state: String,
    pub code: String,
    pub scope: String,
}
#[get("/login/google/redirect")]
pub async fn login_redirect(
    query: web::Query<LoginQuery>,
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let query = query.into_inner();
    let client = oauth_client(&*config);
    web::block(move || {
        let token_result = client.exchange_code(AuthorizationCode::new(query.code));
        dbg!(token_result);
        Ok::<_, ()>(())
    })
    .await
    .unwrap();
    HttpResponse::NotImplemented().body("boi")
}

#[get("/login/google")]
pub async fn login(config: web::Data<Config>) -> impl Responder {
    let client = oauth_client(&*config);
    let (auth_url, csrf_token) = client.authorize_url(CsrfToken::new_random);
    HttpResponse::Found()
        .set_header("Location", auth_url.as_str())
        .finish()
}

#[get("/myself")]
pub async fn myself(pool: web::Data<PgPool>) -> Result<HttpResponse, HttpResponse> {
    let user_id = Uuid::default();
    sqlx::query_as(
        r#"
        SELECT id, display_name, email, created, updated
        FROM user_account
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&**pool)
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
