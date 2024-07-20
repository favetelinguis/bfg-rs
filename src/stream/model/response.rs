// Authentication
// MarketSubscription
// OrderSubscription
// Heartbeat

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResponse {
    connection_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    Success,
    Failure,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    status_code: StatusCode,
    error_code: Option<String>,
    error_message: Option<String>,
    connection_closed: bool,
    connections_available: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum ResponseMessage {
    Connection(ConnectionResponse),
    Status(StatusResponse),
    Mcm(String),
    Ocm(String),
}
