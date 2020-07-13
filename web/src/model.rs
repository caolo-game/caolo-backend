use crate::PgPool;
use anyhow::Context;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

lazy_static! {
    // TODO: RSA
    static ref JWT_SECRET: String = {
        let secret = std::env::var("SECRET").unwrap_or_else(|_| "foobar".to_owned());
        secret
    };
    static ref JWT_ENCODE: EncodingKey = EncodingKey::from_secret(JWT_SECRET.as_bytes());
    static ref JWT_DECODE: DecodingKey<'static> = DecodingKey::from_secret(JWT_SECRET.as_bytes());
}

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum UserReadError {
    #[error("Failed to query the database")]
    DbError(sqlx::Error),
}

impl warp::reject::Reject for UserReadError {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Identity {
    pub id: Uuid,
    pub token: String,
    pub exp: usize,
    pub iat: usize,
}

impl FromStr for Identity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = decode::<Identity>(s, &*JWT_DECODE, &Validation::default())
            .with_context(|| "failed to deseralize identity")?;
        Ok(token.claims)
    }
}

impl Identity {
    pub fn serialize_token(&self) -> Result<String, anyhow::Error> {
        let token = encode(&Header::default(), self, &JWT_ENCODE)
            .with_context(|| "failed to encode identity")?;
        Ok(token)
    }
}

pub async fn current_user(
    id: Option<Identity>,
    pool: PgPool,
) -> Result<Option<User>, warp::Rejection> {
    debug!("LoggedInUser from request {:?}", id);
    let id = match id {
        Some(id) => id,
        None => return Ok(None),
    };
    sqlx::query_as!(
        User,
        "
        SELECT ua.id, ua.display_name, ua.email, ua.created, ua.updated
        FROM user_account AS ua
        INNER JOIN user_credential AS uc
        ON uc.token = $1 AND uc.user_id = ua.id; ",
        id.token
    )
    .fetch_optional(&pool)
    .await
    .map_err(UserReadError::DbError)
    .map_err(warp::reject::custom)
}
