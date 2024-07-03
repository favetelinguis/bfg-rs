use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::blocking::{Client, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE},
    Identity, Method,
};

use serde::Deserialize;

/// Status enum for logut and keep-alive
#[derive(Deserialize, Debug)]
enum AuthStatus {
    SUCCESS,
    FAIL,
}

/// Error enum for logut and keep-alive
#[derive(Deserialize, Debug)]
enum AuthError {
    INPUT_VALIDATION_ERROR,
    INTERNAL_ERROR,
    NO_SESSION,
}

// Represent the response after logging in
#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    token: String,
    product: String,
    status: AuthStatus,
    error: AuthError,
}

#[derive(Deserialize, Debug)]
enum BotLoginStatus {
    SUCCESS,
    ACCOUNT_ALREADY_LOCKED,
    ACCOUNT_NOW_LOCKED,
    ACCOUNT_PENDING_PASSWORD_CHANGE,
    ACTIONS_REQUIRED,
    AGENT_CLIENT_MASTER,
    AGENT_CLIENT_MASTER_SUSPENDED,
    AUTHORIZED_ONLY_FOR_DOMAIN_RO,
    AUTHORIZED_ONLY_FOR_DOMAIN_SE,
    BETTING_RESTRICTED_LOCATION,
    CERT_AUTH_REQUIRED,
    CHANGE_PASSWORD_REQUIRED,
    CLOSED,
    DANISH_AUTHORIZATION_REQUIRED,
    DENMARK_MIGRATION_REQUIRED,
    DUPLICATE_CARDS,
    EMAIL_LOGIN_NOT_ALLOWED,
    INPUT_VALIDATION_ERROR,
    INTERNATIONAL_TERMS_ACCEPTANCE_REQUIRED,
    INVALID_CONNECTIVITY_TO_REGULATOR_DK,
    INVALID_CONNECTIVITY_TO_REGULATOR_IT,
    INVALID_USERNAME_OR_PASSWORD,
    ITALIAN_CONTRACT_ACCEPTANCE_REQUIRED,
    ITALIAN_PROFILING_ACCEPTANCE_REQUIRED,
    KYC_SUSPEND,
    MULTIPLE_USERS_WITH_SAME_CREDENTIAL,
    NOT_AUTHORIZED_BY_REGULATOR_DK,
    NOT_AUTHORIZED_BY_REGULATOR_IT,
    PENDING_AUTH,
    PERSONAL_MESSAGE_REQUIRED,
    SECURITY_QUESTION_WRONG_3X,
    SECURITY_RESTRICTED_LOCATION,
    SELF_EXCLUDED,
    SPAIN_MIGRATION_REQUIRED,
    SPANISH_TERMS_ACCEPTANCE_REQUIRED,
    SUSPENDED,
    SWEDEN_BANK_ID_VERIFICATION_REQUIRED,
    SWEDEN_NATIONAL_IDENTIFIER_REQUIRED,
    TELBET_TERMS_CONDITIONS_NA,
    TEMPORARY_BAN_TOO_MANY_REQUESTS,
    TRADING_MASTER,
    TRADING_MASTER_SUSPENDED,
}

// Represent the response bot login
#[derive(Deserialize, Debug)]
pub struct BotLoginResponse {
    sessionToken: Option<String>,
    loginStatus: BotLoginStatus,
}

/// Login to betfair
pub fn login(url: &str, app_key: &str, username: &str, password: &str) -> Result<BotLoginResponse> {
    // Load the client certificate and key
    let cert = std::fs::read("betfaircert/betfair-2048.crt")
        .with_context(|| "unable to read betfaircert/betfair-2048.crt")?;
    let key = std::fs::read("betfaircert/betfair-2048.key")
        .with_context(|| "unable to read betfaircert/betfair-2048.key")?;

    // Create an identity from the certificate and key
    let identity = Identity::from_pkcs8_pem(&cert, &key)?;

    // Create the client with the identity
    let client = Client::builder()
        .identity(identity)
        .danger_accept_invalid_hostnames(true) // Similar to StrictHostnameVerifier but more dangerous
        .build()?;

    // let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
    headers.insert("X-Application", HeaderValue::from_str(app_key).unwrap());

    let mut params = HashMap::new();
    params.insert("username", username);
    params.insert("password", password);

    client
        .request(
            Method::POST,
            "https://identitysso-cert.betfair.se/api/certlogin",
        )
        .headers(headers)
        .form(&params)
        .send()?
        .json::<BotLoginResponse>()
        .with_context(|| "login failed")
}

/// Keep alive to betfair
pub fn keep_alive(url: &str, app_key: &str, token: &str) -> Result<LoginResponse> {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
    headers.insert("X-Application", HeaderValue::from_str(app_key).unwrap());
    headers.insert("X-Authentication", HeaderValue::from_str(token).unwrap());
    client
        .request(Method::POST, format!("{}/keepAlive", url))
        .headers(headers)
        .send()?
        .json::<LoginResponse>()
        .with_context(|| "keep-alive failed")
}

/// Logout to betfair
pub fn logout(url: &str, app_key: &str, token: &str) -> Result<LoginResponse> {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
    headers.insert("X-Application", HeaderValue::from_str(app_key).unwrap());
    headers.insert("X-Authentication", HeaderValue::from_str(token).unwrap());
    client
        .request(Method::POST, format!("{}/logout", url))
        .headers(headers)
        .send()?
        .json::<LoginResponse>()
        .with_context(|| "logout failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request() {
        let mut server = mockito::Server::new();

        // Mock endpoint
        let mock = server
            .mock("POST", "/login")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{ "token":"SESSION_TOKEN", "product":"APP_KEY", "status":"SUCCESS", "error":"" }"#,
            )
            .create();

        // Perform HTTP request
        let res = login(&server.url(), "key", "user", "pwd").unwrap();

        // Assertions
        mock.assert();
        // assert_eq!(res.status(), 200);
        // let body = res.text().unwrap();
        // assert_eq!(
        //     body,
        //     r#"{ "token":"SESSION_TOKEN", "product":"APP_KEY", "status":"SUCCESS", "error":"" }"#
        // );
    }
}

// To perform the same login to Betfair with client certificates using the `reqwest` library in Rust, you can translate the Java code roughly as follows:

// ```rust
// use reqwest::blocking::{Client, ClientBuilder};
// use reqwest::Error;
// use std::fs::File;
// use std::io::{self, Read};
// use reqwest::Certificate;
// use std::path::Path;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Load the client certificate
//     let mut buf = Vec::new();
//     let mut file = File::open("C:/sslcerts/client-2048.p12")?;
//     file.read_to_end(&mut buf)?;

//     // Create a client builder with the certificate
//     let client = ClientBuilder::new()
//         .identity(reqwest::Identity::from_pkcs12_der(&buf, "password")?)
//         .danger_accept_invalid_hostnames(true)  // Similar to StrictHostnameVerifier but more dangerous
//         .build()?;

//     // Set up login parameters
//     let params = [
//         ("username", "testuser"),
//         ("password", "testpassword"),
//     ];

//     // Make the HTTP POST request
//     let response = client.post("https://identitysso-cert.betfair.com/api/certlogin")
//         .header("X-Application", "appkey")
//         .form(&params)
//         .send()?;

//     // Print the status and response
//     println!("Status: {}", response.status());
//     println!("Response body: {:?}", response.text()?);

//     Ok(())
// }
// ```

// ### Explanations:
// 1. **Loading the Client Certificate:**
//    - The certificate is read from a `.p12` file.
// 2. **Creating the Client:**
//    - The client is built with the loaded certificate. Note that `danger_accept_invalid_hostnames` is used to match the Java `StrictHostnameVerifier`, but this disables hostname verification entirely, which is not recommended for production use.
// 3. **Setting Up Login Parameters:**
//    - Similar to how `NameValuePair` was used in Java, the parameters are formed into an array of tuples.
// 4. **Performing the POST Request:**
//    - The request is sent with the necessary headers and parameters.
// 5. **Printing the Response:**
//    - The status and response body are printed.

// **Note**: Ensure you handle errors and exceptions properly in a production environment, especially when dealing with potential issues in reading files and making network requests.
