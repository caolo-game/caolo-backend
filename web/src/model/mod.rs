use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;
use serde::Serialize;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
