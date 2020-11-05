use slog::{debug, error, o, warn, Drain, Logger};
use std::env::{self, VarError};
use std::net::IpAddr;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Config {
    pub db_url: String,
    pub amqp_url: String,

    pub allowed_origins: Vec<String>,
    pub base_url: String,
    pub host: IpAddr,
    pub port: u16,

    pub auth_token_audience: String,
}

#[derive(Error, Debug)]
pub enum ConfigReadError {
    #[error("Env variable [{0}] could not be read {1:?}")]
    EnvNotSet(String, VarError),
}

impl Config {
    pub fn read(logger: impl Into<Option<Logger>>) -> Result<Self, ConfigReadError> {
        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(slog_stdlog::StdLog.fuse(), o!()));
        debug!(logger, "Reading configuration");
        let host = env::var("HOST")
            .ok()
            .and_then(|host| {
                host.parse()
                    .map_err(|e| {
                        error!(logger, "Failed to parse host {:?}", e);
                    })
                    .ok()
            })
            .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1]));
        let port = env::var("PORT")
            .map_err(anyhow::Error::new)
            .and_then(|port| port.parse().map_err(anyhow::Error::new))
            .unwrap_or_else(|err| {
                warn!(logger, "Failed to parse port number: {}", err);
                8000
            });

        let amqp_url = std::env::var("AMQP_ADDR")
            .or_else(|_| std::env::var("CLOUDAMQP_URL"))
            .unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_owned());

        let config = Config {
            amqp_url,
            allowed_origins: env::var("ALLOWED_ORIGINS")
                .map(|origins| origins.split(";").map(|s| s.to_owned()).collect())
                .unwrap_or_else(|_| vec!["http://localhost:3000".to_owned()]),
            db_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:admin@localhost:5432/caolo".to_owned()),
            host,
            port,
            base_url: env::var("CAOLO_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_owned()),

            auth_token_audience: env::var("TOKEN_AUDIENCE")
                .unwrap_or_else(|_| "http://localhost:8000".to_owned()),
        };
        debug!(logger, "Reading configuration done {:#?}", config);
        Ok(config)
    }
}
