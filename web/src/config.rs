use std::env::{self, VarError};
use thiserror::Error;

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub redis_url: String,
    pub db_url: String,
    pub google_id: String,
    pub google_secret: String,
    pub base_url: String,
    pub host: String,
    pub port: String,
}

#[derive(Error, Debug)]
pub enum ConfigReadError {
    #[error("Env variable [{0}] could not be read {1:?}")]
    EnvNotSet(String, VarError),
}

impl Config {
    pub fn read() -> Result<Self, ConfigReadError> {
        let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_owned());
        let port = env::var("PORT").unwrap_or_else(|_| "8000".to_owned());
        let config = Config {
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|origins| origins.split(";").map(|s|s.to_owned()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:3000".to_owned()]),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/0".to_owned()),
            db_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:admin@localhost:5432/caolo".to_owned()),
            google_id: env::var("GOOGLE_OAUTH_CLIENT_ID")
                .map_err(|e| ConfigReadError::EnvNotSet("GOOGLE_OAUTH_CLIENT_ID".to_owned(), e))?,
            google_secret: env::var("GOOGLE_OAUTH_CLIENT_SECRET").map_err(|e| {
                ConfigReadError::EnvNotSet("GOOGLE_OAUTH_CLIENT_SECRET".to_owned(), e)
            })?,
            host,
            port,
            base_url: env::var("CAOLO_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_owned()),
        };
        Ok(config)
    }
}
