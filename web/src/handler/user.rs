use crate::model::User;
use crate::PgPool;
use serde::Deserialize;
use slog::{error, Logger};
use std::convert::Infallible;
use thiserror::Error;
use warp::http::StatusCode;
use warp::reply::with_status;

pub async fn myself(user: Option<User>) -> Result<impl warp::Reply, Infallible> {
    let resp = warp::reply::json(&user);
    let resp = match user {
        Some(_) => with_status(resp, StatusCode::OK),
        None => with_status(resp, StatusCode::NOT_FOUND),
    };
    Ok(resp)
}

#[derive(Deserialize, Debug)]
pub struct UserRegistrationData {
    pub user_id: String,
    #[serde(rename = "name")]
    pub username: String,
    pub email: String,
}

#[derive(Debug, Error)]
pub enum UserRegistrationError {
    #[error("This user ID has already been registered")]
    UserIdTaken,
    #[error("This e-mail address has already been registered")]
    EmailTaken,
    #[error("Unexpected error while attempting to register new user")]
    InternalError,
}

impl warp::reject::Reject for UserRegistrationError {}

impl UserRegistrationError {
    pub fn status(&self) -> StatusCode {
        match self {
            UserRegistrationError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            UserRegistrationError::UserIdTaken | UserRegistrationError::EmailTaken => {
                StatusCode::BAD_REQUEST
            }
        }
    }
}

pub async fn put_user(
    logger: Logger,
    payload: UserRegistrationData,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = sqlx::query!(
        "
        INSERT INTO user_account
            (auth0_id, display_name, email, email_verified)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT ON CONSTRAINT email_is_unique
        DO UPDATE
        SET 
            display_name=$2,
            email_verified=$4
        ",
        payload.user_id,
        payload.username,
        payload.email,
        true
    )
    .execute(&db)
    .await;
    if let Err(err) = res {
        error!(logger, "Unexpected error while registering user {:?}", err);
        let err = UserRegistrationError::InternalError;
        return Err(warp::reject::custom(err));
    }
    let res = warp::reply::reply();
    Ok(res)
}
pub async fn register(
    logger: Logger,
    payload: UserRegistrationData,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res = sqlx::query!(
        "
        INSERT INTO user_account
            (auth0_id, display_name, email, email_verified)
        VALUES
            ($1, $2, $3, $4)
        ",
        payload.user_id,
        payload.username,
        payload.email,
        true
    )
    .execute(&db)
    .await;
    if let Err(err) = res {
        use sqlx::Error;
        match err {
            Error::Database(ref err) => match err.constraint_name() {
                Some(constraint) if constraint == "email_is_unique" => {
                    let err = UserRegistrationError::EmailTaken;
                    return Err(warp::reject::custom(err));
                }
                Some(constraint) if constraint == "auth0_id_is_unique" => {
                    let err = UserRegistrationError::UserIdTaken;
                    return Err(warp::reject::custom(err));
                }
                _ => {}
            },
            _ => {}
        }
        error!(logger, "Unexpected error while registering user {:?}", err);
        let err = UserRegistrationError::InternalError;
        return Err(warp::reject::custom(err));
    }
    let res = warp::reply::reply();
    Ok(res)
}
