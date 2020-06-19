use crate::google_auth::oauth_client;
use crate::Config;
use crate::PgPool;
use crate::RedisPool;
use actix_identity::Identity;
use actix_web::error::BlockingError;
use actix_web::web::{self, HttpResponse};
use actix_web::{get, http::StatusCode, Responder, ResponseError};
use log::{debug, error};
use oauth2::{prelude::*, AuthorizationCode, CsrfToken, TokenResponse};
use rand::RngCore;
use serde_json::Value;
use std::ops::DerefMut;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginMetadata {
    pub redirect: Option<String>,
    pub csrf_token: CsrfToken,
}

#[derive(Deserialize, Debug)]
pub struct LoginQuery {
    pub redirect: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LoginRedirectQuery {
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
    #[error("There has been an error querying the database")]
    DbError(sqlx::Error),
    #[error("There has been an error querying the cache")]
    CachePoolError(r2d2_redis::r2d2::Error),
    #[error("Failed to authorize via Google")]
    ExchangeCodeFailure,
    #[error("Critical server error")]
    BlockingCancel,
    #[error("Failed to query Google for user info")]
    GoogleMyselfQueryFailure,
    #[error("Failed to deserialize state {0:?}")]
    GoogleMyselfDeserializationError(reqwest::Error),
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::CsrfTokenMisMatch => StatusCode::UNAUTHORIZED,
            Self::CsrfTokenNotFoundInCache => StatusCode::UNAUTHORIZED,
            Self::CsrfTokenDeserializeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::IdNotFoud => StatusCode::UNAUTHORIZED,
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CachePoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BlockingCancel => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ExchangeCodeFailure => StatusCode::UNAUTHORIZED,
            Self::GoogleMyselfQueryFailure => StatusCode::UNAUTHORIZED,
            Self::GoogleMyselfDeserializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[get("/login/google/redirect")]
pub async fn login_redirect(
    query: web::Query<LoginRedirectQuery>,
    identity: Identity,
    config: web::Data<Config>,
    cache: web::Data<RedisPool>,
    db: web::Data<PgPool>,
) -> Result<impl Responder, LoginError> {
    let query = query.into_inner();

    let identity = identity.identity().ok_or_else(|| LoginError::IdNotFoud)?;

    let meta: LoginMetadata = get_csrf_token(cache.into_inner(), identity.clone()).await?;
    if meta.csrf_token.secret() != &query.state {
        error!(
            "Got invalid csrf_token expected: {:?}, found: {:?}",
            meta.csrf_token, query.state
        );
        return Err(LoginError::CsrfTokenMisMatch);
    }

    let client = oauth_client(&*config);
    let token = web::block(move || client.exchange_code(AuthorizationCode::new(query.code)))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => {
                error!("Failed to exchange code {:?}", e);
                LoginError::ExchangeCodeFailure
            }
            BlockingError::Canceled => LoginError::BlockingCancel,
        })?;

    let access_token = token.access_token();

    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/plus/v1/people/me")
        .bearer_auth(access_token.secret())
        .send()
        .await
        .map_err(|e| {
            error!("Error while getting user info {:?}", e);
            LoginError::GoogleMyselfQueryFailure
        })?;
    if response.status() != 200 {
        error!("Google response {:#?}", response);
        return Err(LoginError::GoogleMyselfQueryFailure);
    }
    let json: Value = response.json().await.map_err(|e| {
        error!("Error getting response body {:?}", e);
        LoginError::GoogleMyselfDeserializationError(e)
    })?;
    debug!("Google response {:#?}", json);
    let google_id = json["id"].as_str().expect("Id not found in user data");
    let email: Option<&str> = json["emails"]
        .get(0)
        .and_then(|email| email["value"].as_str());

    let db = db.into_inner();

    let user_id: Option<Uuid> = sqlx::query!(
        "
        SELECT user_id FROM user_google_token
        WHERE google_id = $1;
        ",
        google_id
    )
    .fetch_optional(&*db)
    .await
    .map_err(LoginError::DbError)?
    .map(|row| row.user_id);

    let user_id = match user_id {
        Some(x) => {
            sqlx::query!(
                "
                INSERT INTO user_credential (user_id, token)
                VALUES ($1, $2)
                ON CONFLICT (user_id) DO
                UPDATE 
                SET token = $2
                ",
                user_id,
                identity
            )
            .execute(&*db)
            .await
            .map_err(LoginError::DbError)?;
            x
        }
        None => register_user(Arc::clone(&db), email, identity.as_str()).await?,
    };

    sqlx::query!(
        "
        INSERT INTO user_google_token (google_id, user_id, access_token)
        VALUES ($1,$2,$3)
        ON CONFLICT (google_id, user_id) DO
        UPDATE 
        SET access_token=$3
        ",
        google_id,
        user_id,
        access_token.secret()
    )
    .execute(&*db)
    .await
    .map_err(LoginError::DbError)?;

    let response = meta
        .redirect
        .map(|redirect| {
            HttpResponse::Found()
                .set_header("Location", redirect)
                .finish()
        })
        .unwrap_or_else(|| HttpResponse::Ok().finish());

    Ok(response)
}

async fn get_csrf_token(
    cache: Arc<RedisPool>,
    identity: String,
) -> Result<LoginMetadata, LoginError> {
    let mut conn = cache.get().map_err(LoginError::CachePoolError)?;
    web::block(move || {
        redis::pipe()
            .get(&identity)
            .del(&identity)
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
            BlockingError::Error(e) => e,
            BlockingError::Canceled => LoginError::BlockingCancel,
        }
    })
}

async fn register_user(
    db: Arc<PgPool>,
    email: Option<&str>,
    identity: &str,
) -> Result<Uuid, LoginError> {
    let mut tx = db.begin().await.map_err(LoginError::DbError)?;
    let user_id = sqlx::query!(
        "INSERT INTO user_account (email)
                VALUES ($1)
                RETURNING id;",
        email
    )
    .fetch_one(&mut tx)
    .await
    .map_err(LoginError::DbError)?
    .id;

    sqlx::query!(
        "
                INSERT INTO user_credential (user_id, token)
                VALUES ($1, $2)
                ",
        user_id,
        identity
    )
    .execute(&mut tx)
    .await
    .map_err(LoginError::DbError)?;

    tx.commit().await.map_err(LoginError::DbError)?;

    Ok(user_id)
}

#[get("/login/google")]
pub async fn login(
    query: web::Query<LoginQuery>,
    config: web::Data<Config>,
    id: Identity,
    cache: web::Data<RedisPool>,
) -> Result<impl Responder, HttpResponse> {
    let mut rng = rand::thread_rng();
    let mut randid = vec![0; 128];
    rng.fill_bytes(&mut randid);

    let randid = base64::encode(&randid);
    let client = oauth_client(&*config);

    let (auth_url, csrf_token) = client.authorize_url(CsrfToken::new_random);

    let mut conn = cache
        .into_inner()
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let meta = LoginMetadata {
        csrf_token,
        redirect: query.into_inner().redirect,
    };

    let meta = serde_json::to_string(&meta).unwrap();
    id.remember(randid.clone());
    web::block(move || {
        redis::pipe()
            .set_ex(&randid, meta, 60)
            .query(conn.deref_mut())
    })
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let result = HttpResponse::Found()
        .set_header("Location", auth_url.as_str())
        .finish();
    Ok(result)
}
