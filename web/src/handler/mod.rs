use crate::google_auth::oauth_client;
use crate::model::User;
use crate::Config;
use crate::PgPool;
use crate::RedisPool;
use actix_identity::Identity;
use actix_web::error::BlockingError;
use actix_web::web::{self, HttpResponse, Json};
use actix_web::{error, get, post, Responder};
use cao_lang::compiler::{self, CompilationUnit};
use log::{debug, error};
use oauth2::{prelude::*, AuthorizationCode, CsrfToken};
use rand::RngCore;
use serde::Deserialize;
use sqlx::postgres::PgQueryAs;
use std::ops::DerefMut;
use thiserror::Error;
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

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Failed to confirm identity")]
    CsrfTokenMisMatch,
    #[error("Login state was not found")]
    CsrfTokenNotFoundInCache,
    #[error("Failed to deserialize state {0:?}")]
    CsrfTokenDeserializeError(serde_json::Error),
    #[error("Identity was not found.")]
    IdNotFoud,
}

#[get("/login/google/redirect")]
pub async fn login_redirect(
    query: web::Query<LoginQuery>,
    id: Identity,
    config: web::Data<Config>,
    cache: web::Data<RedisPool>,
    db: web::Data<PgPool>,
) -> Result<impl Responder, HttpResponse> {
    let query = query.into_inner();

    let id = id.identity().ok_or_else(|| {
        debug!("login_redirect called without an identity");
        HttpResponse::NotFound().body(format!("{}", LoginError::IdNotFoud))
    })?;

    let mut conn = cache
        .into_inner()
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let csrf_token: CsrfToken = web::block(move || {
        redis::pipe()
            .get(&id)
            .del(&id)
            .ignore()
            .query(conn.deref_mut())
            .map_err(|e| {
                error!("Failed read csrf_token {:?}", e);
                LoginError::CsrfTokenNotFoundInCache
            })
            .and_then(|v: Vec<String>| {
                v.get(0)
                    .ok_or_else(|| LoginError::CsrfTokenNotFoundInCache)
                    .and_then(|v| {
                        serde_json::from_str(v.as_str())
                            .map_err(LoginError::CsrfTokenDeserializeError)
                    })
            })
    })
    .await
    .map_err(|e| {
        error!("Failed to get csrf_token {:?}", e);
        match e {
            BlockingError::Error(LoginError::CsrfTokenNotFoundInCache) => {
                HttpResponse::NotFound().body(format!("{}", LoginError::CsrfTokenNotFoundInCache))
            }
            BlockingError::Error(LoginError::CsrfTokenDeserializeError(e)) => {
                HttpResponse::InternalServerError().body(format!("{:?}", e))
            }
            BlockingError::Canceled => HttpResponse::InternalServerError().finish(),
            _ => unreachable!(),
        }
    })?;

    if csrf_token.secret() != &query.state {
        error!(
            "Got invalid csrf_token expected: {:?}, found: {:?}",
            csrf_token, query.state
        );
        return Err(HttpResponse::Unauthorized().body(format!("{}", LoginError::CsrfTokenMisMatch)));
    }

    let client = oauth_client(&*config);
    let token = web::block(move || client.exchange_code(AuthorizationCode::new(query.code)))
        .await
        .map_err(actix_web::error::ErrorUnauthorized)?;

    // TODO: register the user if not exists
    // TODO: save the token

    // sqlx::query!(
    //     "
    //     UPDATE user_credential
    //     SET token=$1
    //     WHERE user_id=$2
    //     "
    //     ,token,
    //     user_id
    // )
    //     .execute(db).await.map_err(actix_web::error::ErrorInternalServerError)?;

    let response = HttpResponse::Ok().body("boi");
    Ok(response)
}

#[get("/login/google")]
pub async fn login(
    config: web::Data<Config>,
    id: Identity,
    cache: web::Data<RedisPool>,
) -> Result<impl Responder, HttpResponse> {
    let mut rng = rand::thread_rng();
    let mut randid = vec![0; 128];
    rng.fill_bytes(&mut randid);

    let randid = randid.into_iter().map(|c| c as char).collect::<String>();
    let client = oauth_client(&*config);

    let (auth_url, csrf_token) = client.authorize_url(CsrfToken::new_random);

    let mut conn = cache
        .into_inner()
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let csrf_token = serde_json::to_string(&csrf_token).unwrap();
    id.remember(randid.clone());
    web::block(move || {
        redis::pipe()
            .set_ex(&randid, csrf_token, 60)
            .query(conn.deref_mut())
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let result = HttpResponse::Found()
        .set_header("Location", auth_url.as_str())
        .finish();
    Ok(result)
}

#[get("/myself")]
pub async fn myself(db: web::Data<PgPool>) -> Result<HttpResponse, HttpResponse> {
    let db = db.into_inner();
    let user_id = Uuid::default();
    sqlx::query_as(
        r#"
        SELECT id, display_name, email, created, updated
        FROM user_account
        WHERE id = $1
        "#,
    )
    .bind(user_id)
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
