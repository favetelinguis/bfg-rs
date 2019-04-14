//!
//! Holds the core logic and traits for bfg
//!
//!
pub struct Config {
    pub username: String,
    pub password: String,
    pub account: String,
    pub api_key: String,
}

impl Config {
    pub fn new(username: String, password: String, account: String, api_key: String) -> Config {
        Config {
            username,
            password,
            account,
            api_key,
        }
    }
}
