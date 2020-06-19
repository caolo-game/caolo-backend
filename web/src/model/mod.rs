use crate::PgPool;
use serde::{Serialize};
use actix_identity::Identity;
use actix_web::FromRequest;
use actix_web::{
    dev::Payload,
    http::StatusCode,
    web::{self, HttpRequest},
    ResponseError,
};
use chrono::{DateTime, Utc};
use log::debug;
use sqlx::FromRow;
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use uuid::Uuid;

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
    #[error("Failed to connect to database")]
    PoolError,
    #[error("Failed to query the database")]
    DbError(sqlx::Error),
    #[error("Identity could not be found")]
    IdNotFound,
}

impl ResponseError for UserReadError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::IdNotFound => StatusCode::NOT_FOUND,
            Self::PoolError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl FromRequest for User {
    type Error = UserReadError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        debug!("LoggedInUserId from_request");
        let id = Identity::from_request(req, payload);
        let conn = web::Data::<PgPool>::from_request(req, payload);
        Box::pin(async move {
            let id = id.await.map_err(|e| {
                debug!("Id not found {:?}", e);
                UserReadError::IdNotFound
            })?;
            let conn = conn.await.map_err(|e| {
                debug!("Conn not found {:?}", e);
                UserReadError::PoolError
            })?;
            sqlx::query_as!(
                User,
                "
                SELECT ua.id, ua.display_name, ua.email, ua.created, ua.updated
                FROM user_account AS ua
                INNER JOIN user_credential AS uc
                ON uc.token = $1 AND uc.user_id = ua.id;
                ",
                id.identity()
            )
            .fetch_one(&*conn.into_inner())
            .await
            .map_err(UserReadError::DbError)
        })
    }
}
