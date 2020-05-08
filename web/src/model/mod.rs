use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
