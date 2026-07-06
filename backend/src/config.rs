use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiration_secs: i64,
    pub refresh_token_expiration_secs: i64,
    pub issuer: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub log_format: String,
    pub enable_metrics: bool,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

        let config = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("database.max_connections", 10)?
            .set_default("jwt.access_token_expiration_secs", 900)?
            .set_default("jwt.refresh_token_expiration_secs", 604800)?
            .set_default("jwt.issuer", "halo-server")?
            .set_default("observability.log_level", "info")?
            .set_default("observability.log_format", "pretty")?
            .set_default("observability.enable_metrics", false)?
            .add_source(
                config::File::with_name(&format!("config/{}", env))
                    .required(false),
            )
            .add_source(
                config::Environment::default()
                    .prefix("HALO")
                    .separator("__"),
            )
            .build()?;

        config.try_deserialize()
    }
}
