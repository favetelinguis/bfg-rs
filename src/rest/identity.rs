use color_eyre::eyre::{self, Context};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT},
    Method,
};
use serde::Deserialize;

const IDENTITY_URL: &str = "https://identitysso.betfair.se/api";

// // FIX example how to use time in json serde
// #[derive(Deserialize, Debug)]
// struct ExampleResponseWithDate {
//     datestamp: DateTime<Utc>,
// }

#[derive(Deserialize, Debug)]
pub struct IdentityResponse {
    token: String,
    product: String,
    status: IdentityStatus,
    error: IdentityError,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum IdentityError {
    InputValidationError,
    InternalError,
    NoSession,
}

/// Status enum for logut and keep-alive
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum IdentityStatus {
    Success,
    Fail,
}

/// Keep alive to betfair
pub fn keep_alive(app_key: &str, token: &str) -> eyre::Result<IdentityResponse> {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
    headers.insert("X-Application", HeaderValue::from_str(app_key).unwrap());
    headers.insert("X-Authentication", HeaderValue::from_str(token).unwrap());
    client
        .request(Method::POST, format!("{}/keepAlive", IDENTITY_URL))
        .headers(headers)
        .send()?
        .json::<IdentityResponse>()
        .wrap_err("keep-alive failed")
}

pub fn logout(app_key: &str, token: &str) -> eyre::Result<IdentityResponse> {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
    headers.insert("X-Application", HeaderValue::from_str(app_key).unwrap());
    headers.insert("X-Authentication", HeaderValue::from_str(token).unwrap());
    client
        .request(Method::POST, format!("{}/logout", IDENTITY_URL))
        .headers(headers)
        .send()?
        .json::<IdentityResponse>()
        .wrap_err("logout failed")
}
