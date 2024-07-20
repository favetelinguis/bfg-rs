use color_eyre::eyre::{self, Context};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, ACCEPT},
    Identity, Method,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

const LOGIN_URL: &str = "https://identitysso-cert.betfair.se/api";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum LoginStatus {
    Success,
    AccountAlreadyLocked,
    AccountNowLocked,
    AccountPendingPasswordChange,
    ActionsRequired,
    AgentClientMaster,
    AgentClientMasterSuspended,
    AuthorizedOnlyForDomainRo,
    AuthorizedOnlyForDomainSe,
    BettingRestrictedLocation,
    CertAuthRequired,
    ChangePasswordRequired,
    Closed,
    DanishAuthorizationRequired,
    DenmarkMigrationRequired,
    DuplicateCards,
    EmailLoginNotAllowed,
    InputValidationError,
    InternationalTermsAcceptanceRequired,
    InvalidConnectivityToRegulatorDk,
    InvalidConnectivityToRegulatorIt,
    InvalidUsernameOrPassword,
    ItalianContractAcceptanceRequired,
    ItalianProfilingAcceptanceRequired,
    KycSuspend,
    MultipleUsersWithSameCredential,
    NotAuthorizedByRegulatorDk,
    NotAuthorizedByRegulatorIt,
    PendingAuth,
    PersonalMessageRequired,
    SecurityQuestionWrong3x,
    SecurityRestrictedLocation,
    SelfExcluded,
    SpainMigrationRequired,
    SpanishTermsAcceptanceRequired,
    Suspended,
    SwedenBankIdVerificationRequired,
    SwedenNationalIdentifierRequired,
    TelbetTermsConditionsNa,
    TemporaryBanTooManyRequests,
    TradingMaster,
    TradingMasterSuspended,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub session_token: Option<String>,
    pub login_status: LoginStatus,
}

pub fn login(
    app_key: &str,
    username: &str,
    password: &str,
    mut config_dir: PathBuf,
) -> eyre::Result<LoginResponse> {
    // Client cert
    let cert = std::fs::read(config_dir.join("betfair-2048.crt"))
        .wrap_err("unable to read betfair-2048.crt")?;
    let key = std::fs::read(config_dir.join("betfair-2048.key"))
        .wrap_err("unable to read betfair-2048.key")?;

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
        .request(Method::POST, format!("{}/certlogin", LOGIN_URL))
        .headers(headers)
        .form(&params)
        .send()?
        .json::<LoginResponse>()
        .wrap_err("login failed")
}
