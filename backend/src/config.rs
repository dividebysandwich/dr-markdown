use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub allow_registration: bool,
    pub server_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite:./documents.db".to_string(),
            jwt_secret: "your-secret-key-change-this-in-production".to_string(),
            allow_registration: true,
            server_port: 3001,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Override with environment variables if present
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        }

        if let Ok(allow_reg) = std::env::var("ALLOW_REGISTRATION") {
            config.allow_registration = allow_reg.parse().unwrap_or(true);
        }

        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server_port = port.parse().unwrap_or(3001);
        }

        Ok(config)
    }
}