use log::debug;
use std::env::{self, VarError};
use std::net::IpAddr;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub redis_url: String,
    pub db_url: String,
    pub google_id: String,
    pub google_secret: String,
    pub base_url: String,
    pub host: IpAddr,
    pub port: u16,

    pub auth_token_duration: chrono::Duration,
}

#[derive(Error, Debug)]
pub enum ConfigReadError {
    #[error("Env variable [{0}] could not be read {1:?}")]
    EnvNotSet(String, VarError),
}

impl Config {
    pub fn read() -> Result<Self, ConfigReadError> {
        debug!("Reading configuration");
        let host = env::var("HOST")
            .ok()
            .and_then(|host| {
                host.parse()
                    .map_err(|e| {
                        log::error!("Failed to parse host {:?}", e);
                    })
                    .ok()
            })
            .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1]));
        let port = env::var("PORT")
            .map_err(anyhow::Error::new)
            .and_then(|port| port.parse().map_err(anyhow::Error::new))
            .unwrap_or_else(|err| {
                log::warn!("Failed to parse port number: {}", err);
                8000
            });
        let config = Config {
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|origins| origins.split(";").map(|s| s.to_owned()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:3000".to_owned()]),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/0".to_owned()),
            db_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:admin@localhost:5432/caolo".to_owned()),
            google_id: env::var("GOOGLE_OAUTH_CLIENT_ID")
                .map(|id| id.trim().to_owned())
                .map_err(|e| ConfigReadError::EnvNotSet("GOOGLE_OAUTH_CLIENT_ID".to_owned(), e))?,
            google_secret: env::var("GOOGLE_OAUTH_CLIENT_SECRET")
                .map(|id| id.trim().to_owned())
                .map_err(|e| {
                    ConfigReadError::EnvNotSet("GOOGLE_OAUTH_CLIENT_SECRET".to_owned(), e)
                })?,
            host,
            port,
            base_url: env::var("CAOLO_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_owned()),

            auth_token_duration: chrono::Duration::minutes(10),
        };
        debug!("Reading configuration done {:#?}", config);
        Ok(config)
    }
}
