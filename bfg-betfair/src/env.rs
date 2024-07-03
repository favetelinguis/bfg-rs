use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ConnectionConfig {
    pub app_key: String,
    pub password: String,
    pub url: String,
    pub username: String,
}

impl ConnectionConfig {
    pub fn new() -> Result<Self> {
        let url = std::env::var("BFG_URL").with_context(|| "BFG_URL not set in env")?;
        let username =
            std::env::var("BFG_USERNAME").with_context(|| "BFG_USERNAME not set in env")?;
        let password =
            std::env::var("BFG_PASSWORD").with_context(|| "BFG_PASSWORD not set in env")?;
        let app_key = std::env::var("BFG_APP_KEY").with_context(|| "BFG_APP_KEY not set in env")?;

        Ok(ConnectionConfig {
            url,
            username,
            password,
            app_key,
        })
    }
}

// TODO handle reading file from .config to get active session
