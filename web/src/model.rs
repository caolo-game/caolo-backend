use crate::config::Config;
use crate::PgPool;
pub use alcoholic_jwt::{JWK, JWKS};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use slog::{debug, info, trace, warn, Logger};
use sqlx::FromRow;
use std::convert::Infallible;
use std::sync::{Arc, Once, RwLock};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub auth0_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Identity {
    pub id: String,
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Identity {
    /// Returns None on error and logs it.
    pub fn validated_id(
        logger: &Logger,
        config: &Config,
        token: &str,
        jwks: &JWKS,
    ) -> Option<Self> {
        trace!(logger, "deseralizing Identity: {:?}", token);
        let kid = alcoholic_jwt::token_kid(&token)
            .expect("failed to find token")
            .expect("token was empty");
        let jwk = jwks.find(kid.as_str())?;
        let validations = vec![alcoholic_jwt::Validation::Audience(
            config.auth_token_audience.clone(),
        )];
        let token = alcoholic_jwt::validate(&token, jwk, validations)
            .map_err(|e| {
                warn!(logger, "token deserialization failed {:?}", e);
            })
            .ok()?;
        serde_json::from_value(token.claims)
            .map_err(|err| warn!(logger, "failed to deserialize claims {:?}", err))
            .ok()
    }
}

pub async fn current_user(id: Option<Identity>, pool: PgPool) -> Result<Option<User>, Infallible> {
    let id = match id {
        Some(id) => id,
        None => return Ok(None),
    };
    let res = sqlx::query_as!(
        User,
        "
        SELECT ua.id, ua.auth0_id, ua.display_name, ua.email, ua.created, ua.updated
        FROM user_account AS ua
        WHERE ua.auth0_id=$1
        ",
        id.id
    )
    .fetch_optional(&pool)
    .await
    .expect("failed to query database");
    Ok(res)
}

static JWKS_LOAD: Once = Once::new();

pub async fn load_jwks<'a>(
    logger: Logger,
    cache: Arc<RwLock<std::mem::MaybeUninit<JWKS>>>,
) -> Result<&'a JWKS, Infallible> {
    {
        let cache = Arc::clone(&cache);
        tokio::task::spawn_blocking(move || {
            JWKS_LOAD.call_once(|| {
                info!(logger, "performing initial JWK load");
                let cc = Arc::clone(&cache);
                let cache = cc;
                let uri = std::env::var("JWKS_URI")
                    .expect("Can not perform authorization without JWKS_URI");
                let payload = reqwest::blocking::get(&uri);
                let payload = payload.unwrap();
                let payload: JWKS = payload.json().unwrap();

                let mut cache = cache.write().unwrap();
                *cache = std::mem::MaybeUninit::new(payload);
                info!(logger, "JWK load finished");
                debug!(logger, "JWKs loaded: {:#?}", *cache);
            });
        })
        .await
        .expect("Failed to load JWKS");
    }

    let cache = cache.read().unwrap();
    let cache = cache.as_ptr();
    unsafe { Ok(&*cache) }
}
