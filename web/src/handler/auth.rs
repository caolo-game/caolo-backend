use crate::auth::generate_refresh_token;
use crate::google_auth::oauth_client;
use crate::model::Identity;
use crate::Config;
use crate::PgPool;
use crate::RedisPool;
use chrono::Utc;
use log::{debug, error, warn};
use oauth2::reqwest::async_http_client;
use oauth2::AsyncCodeTokenRequest;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, TokenResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::ops::DerefMut;
use thiserror::Error;
use uuid::Uuid;
use warp::http::StatusCode;

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginMetadata {
    pub redirect: Option<String>,
    pub csrf_token: CsrfToken,
    pub pkce_code_verifier: PkceCodeVerifier,
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
    #[error("Failed to authorize via Google")]
    ExchangeCodeFailure,
    #[error("Failed to query Google for user info")]
    GoogleMyselfQueryFailure,
    #[error("Failed to deserialize state {0:?}")]
    GoogleMyselfDeserializationError(reqwest::Error),
}

impl warp::reject::Reject for LoginError {}

impl LoginError {
    pub fn into_reply(&self) -> impl warp::Reply {
        let code = match self {
            LoginError::CsrfTokenMisMatch | LoginError::ExchangeCodeFailure => {
                StatusCode::UNAUTHORIZED
            }
            LoginError::GoogleMyselfQueryFailure => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        warp::reply::with_status(warp::reply::html(format!("{}", self)), code)
    }
}

pub async fn login_redirect(
    session_id: String,
    query: LoginRedirectQuery,
    config: std::sync::Arc<Config>,
    cache: RedisPool,
    db: PgPool,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    login_redirect_impl(session_id, query, config, cache, db)
        .await
        .or_else(|err| {
            match err {
                _ => {
                    error!("Internal error while processing google login {:?}", err);
                }
            }
            let payload = err.into_reply();
            Ok(Box::new(payload))
        })
}

async fn login_redirect_impl(
    identity: String,
    query: LoginRedirectQuery,
    config: std::sync::Arc<Config>,
    cache: RedisPool,
    db: PgPool,
) -> Result<Box<dyn warp::Reply>, LoginError> {
    let meta: LoginMetadata = get_csrf_token(&cache, identity).await?;
    if meta.csrf_token.secret() != &query.state {
        error!(
            "Got invalid csrf_token expected: {:?}, found: {:?}",
            meta.csrf_token, query.state
        );
        return Err(LoginError::CsrfTokenMisMatch);
    }

    let client = oauth_client(&config);
    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(meta.pkce_code_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|err| {
            warn!("Failed to exchange code {:?}", err);
            LoginError::ExchangeCodeFailure
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

    let user_id: Option<Uuid> = sqlx::query!(
        "
        SELECT user_id FROM user_google_token
        WHERE google_id = $1;
        ",
        google_id
    )
    .fetch_optional(&db)
    .await
    .expect("failed to get user_id using the google_id")
    .map(|row| row.user_id);

    let refresh_token = generate_refresh_token(128);

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
                refresh_token.as_str()
            )
            .execute(&db)
            .await
            .expect("failed to insert/update user_credentials");
            x
        }
        None => register_user(&db, email, refresh_token.as_str()).await?,
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
    .execute(&db)
    .await
    .expect("failed to insert/update user_google_token");

    let identity = Identity {
        user_id,
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + chrono::Duration::minutes(5)).timestamp(),
    };

    let response: Box<dyn warp::Reply> = match meta.redirect {
        Some(redirect) => {
            let resp = warp::redirect(redirect.as_str().parse::<warp::http::Uri>().unwrap());
            let resp = set_identity(resp, identity);
            Box::new(resp)
        }
        None => {
            let resp = warp::reply();
            let resp = set_identity(resp, identity);
            Box::new(resp)
        }
    };
    Ok(response)
}

pub fn set_identity(response: impl warp::Reply, identity: Identity) -> impl warp::Reply {
    warp::reply::with_header(
        response,
        "Set-Cookie",
        format!(
            "authorization={}; HttpOnly Secure; Path=/",
            identity.serialize_token().expect("Failed to serialize JWT")
        ),
    )
}

async fn get_csrf_token(cache: &RedisPool, identity: String) -> Result<LoginMetadata, LoginError> {
    let mut conn = cache.get().expect("failed to aquire cache connection");
    tokio::spawn(async move {
        redis::pipe()
            .get(&identity)
            .del(&identity)
            .ignore()
            .query(conn.deref_mut())
            .map_err(|err| {
                error!("Failed read csrf_token {:?}", err);
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
    .expect("Failed to get csrf_token")
    .map_err(|err| {
        warn!("Failed to get csrf_token {:?}", err);
        err
    })
}

async fn register_user(db: &PgPool, email: Option<&str>, token: &str) -> Result<Uuid, LoginError> {
    let mut tx = db.begin().await.expect("failed to begin transaction");
    let user_id = sqlx::query!(
        "INSERT INTO user_account (email)
        VALUES ($1)
        RETURNING id;",
        email
    )
    .fetch_one(&mut tx)
    .await
    .expect("failed to insert into user_account")
    .id;

    sqlx::query!(
        "
        INSERT INTO user_credential (user_id, token)
        VALUES ($1, $2)",
        user_id,
        token
    )
    .execute(&mut tx)
    .await
    .expect("failed to insert into user_credential");

    tx.commit().await.expect("failed to commit transaction");

    Ok(user_id)
}

pub async fn login(
    query: LoginQuery,
    config: std::sync::Arc<Config>,
    cache: RedisPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("user is logging in via Google OAuth");
    let randid = generate_refresh_token(64);

    let client = oauth_client(&*config);

    let (pkce_code_challenge, pkce_code_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("email".to_owned()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let mut conn = cache.get().expect("Failed to aquire cache connection");

    let meta = LoginMetadata {
        csrf_token,
        redirect: query.redirect,
        pkce_code_verifier,
    };

    let meta = serde_json::to_string(&meta).unwrap();

    {
        let randid = randid.clone();
        tokio::spawn(async move {
            redis::pipe()
                .set_ex(&randid, meta, 60)
                .query::<()>(conn.deref_mut())
                .expect("Failed to save csrf challenge in redis");
            Ok::<_, Infallible>(())
        })
        .await
        .expect("Failed to save csrf challenge")
        .unwrap();
    }

    let result = warp::redirect(
        auth_url
            .to_string()
            .parse::<warp::http::Uri>()
            .expect("Expected valid auth url"),
    );
    let result = warp::reply::with_header(
        result,
        "Set-Cookie",
        format!("session_id={}; HttpOnly Secure; Path=/", randid),
    );
    Ok(result)
}
