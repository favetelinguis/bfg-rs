use crate::realtime::models::Direction;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiResponse(pub String);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: String,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateDeletePositionResponse {
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionResponse {
    #[serde(rename = "oauthToken")]
    pub oauth_token: AccessTokenResponse,
    #[serde(rename = "lightstreamerEndpoint")]
    pub lightstreamer_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetMarketResponse {
    pub instrument: InstrumentDetails,
    #[serde(rename = "dealingRules")]
    pub dealing_rules: DealingRules,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Unit {
    PERCENTAGE,
    POINTS,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DealingRule {
    pub unit: Unit,
    pub value: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DealingRules {
    #[serde(rename = "minNormalStopOrLimitDistance")]
    pub min_normal_stop_or_limit_distance: DealingRule,
    #[serde(rename = "minDealSize")]
    pub min_deal_size: DealingRule,
    #[serde(rename = "minStepDistance")]
    pub min_step_distance: DealingRule,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InstrumentDetails {
    pub name: String,
    #[serde(rename = "valueOfOnePip")]
    pub value_of_one_pip: String,
    #[serde(rename = "onePipMeans")]
    pub one_pip_means: String,
    #[serde(rename = "contractSize")]
    pub contract_size: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CurrencyCode {
    EUR,
    USD,
    SEK,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum OrderType {
    LIMIT,
    MARKET,
    QUOTE,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClosePositionRequest {
    pub direction: Direction,
    pub epic: String,
    pub expiry: String,
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    pub size: u8,
}

impl ClosePositionRequest {
    pub fn new(direction: Direction, size: u8) -> Self {
        Self {
            direction,
            order_type: OrderType::MARKET,
            epic: "IX.D.DAX.IFMM.IP".to_string(),
            expiry: "-".to_string(),
            size,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenPositionRequest {
    pub epic: String,         //"IX.D.DAX.IFMM.IP",
    pub expiry: String,       //"-",
    pub direction: Direction, //"SELL",
    pub size: u8,             //"1",
    #[serde(rename = "orderType")]
    pub order_type: OrderType, //"LIMIT",
    pub level: f64,           //"14055",
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool, //"false",
    #[serde(rename = "stopDistance")]
    pub stop_distance: u8, //"10",
    #[serde(rename = "forceOpen")]
    pub force_open: bool, //"true",
    #[serde(rename = "limitDistance")]
    pub limit_distance: u8, //"10",
    #[serde(rename = "currencyCode")]
    pub currency_code: CurrencyCode, //"EUR"
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
}

impl Default for OpenPositionRequest {
    fn default() -> Self {
        Self {
            deal_reference: "CHANGEME".to_string(),
            epic: "IX.D.DAX.IFMM.IP".to_string(),
            expiry: "-".to_string(),
            direction: Direction::BUY,
            size: 1,
            order_type: OrderType::LIMIT,
            level: 0.,
            guaranteed_stop: false,
            stop_distance: 5,
            limit_distance: 5,
            force_open: true,
            currency_code: CurrencyCode::EUR,
        }
    }
}

impl OpenPositionRequest {
    pub fn new(direction: Direction, level: f64, deal_reference: String) -> Self {
        Self {
            direction,
            level,
            deal_reference,
            ..OpenPositionRequest::default()
        }
    }
}

impl From<bfg_core::models::Direction> for Direction {
    fn from(input: bfg_core::models::Direction) -> Self {
        match input {
            bfg_core::models::Direction::SELL => Direction::SELL,
            bfg_core::models::Direction::BUY => Direction::BUY,
        }
    }
}
