use crate::realtime::models::Direction;
use chrono::{NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use bfg_core::decider::MarketInfo;

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
pub struct CreateSessionResponseV2 {
    #[serde(rename = "accountInfo")]
    pub account_info: AccountInfo,
    #[serde(rename = "accountType")]
    pub account_type: AccountType,
    pub accounts: Vec<AccountDetails>,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "currencyIsoCode")]
    pub currency_iso_code: String,
    #[serde(rename = "currencySymbol")]
    pub currency_symbol: String,
    #[serde(rename = "currentAccountId")]
    pub current_account_id: String,
    #[serde(rename = "dealingEnabled")]
    pub dealing_enabled: bool,
    #[serde(rename = "hasActiveDemoAccounts")]
    pub has_active_demo_accounts: bool,
    #[serde(rename = "hasActiveLiveAccounts")]
    pub has_active_live_accounts: bool,
    #[serde(rename = "lightstreamerEndpoint")]
    pub lightstreamer_endpoint: String,
    #[serde(rename = "reroutingEnvironment")]
    pub rerouting_environment: Option<ReroutingEnvironment>,
    #[serde(rename = "timezoneOffset")]
    pub timezone_offset: usize,
    #[serde(rename = "trailingStopsEnabled")]
    pub trailing_stops_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountDetails {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "accountName")]
    account_name: String,
    #[serde(rename = "accountType")]
    account_type: AccountType,
    preferred: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AccountType {
    CFD,
    PHYSICAL,
    SPREADSHEET,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountInfo {
    available: f64,
    balance: f64,
    deposit: f64,
    #[serde(rename = "profitLoss")]
    profit_loss: f64,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ReroutingEnvironment {
    DEMO,
    LIVE,
    TEST,
    UAT,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionRequestV2 {
    pub identifier: String,
    pub password: String,
    #[serde(rename = "encryptedPassword")]
    pub encrypted_password: bool,
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
pub enum WorkingOrderType {
    LIMIT,
    STOP,
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
pub struct EditPositionRequest {
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
    #[serde(rename = "limitLevel")]
    pub limit_level: Option<f64>,
    #[serde(rename = "stopLevel")]
    pub stop_level: f64,
    #[serde(rename = "trailingStop")]
    pub trailing_stop: bool,
    #[serde(rename = "trailingStopDistance")]
    pub trailing_stop_distance: u8,
    #[serde(rename = "trailingStopIncrement")]
    pub trailing_stop_increment: u8,
}

impl Default for EditPositionRequest {
    fn default() -> Self {
        Self {
            guaranteed_stop: false,
            stop_level: 0.,
            limit_level: None,
            trailing_stop_distance: 0,
            trailing_stop_increment: 1, // 1 looks to be the min level which is not ideal for markets that move little
            trailing_stop: true,
        }
    }
}

impl EditPositionRequest {
    pub fn new(stop_level: f64, trailing_stop_distance: u8, target_level: Option<f64>) -> Self {
        Self {
            stop_level,
            trailing_stop_distance,
            limit_level: target_level,
            ..EditPositionRequest::default()
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateWorkingOrderRequest {
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
    pub direction: Direction,
    pub epic: String,
    pub expiry: String,
    #[serde(rename = "forceOpen")]
    pub force_open: bool,
    #[serde(rename = "goodTillDate")]
    pub good_till_date: String,
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
    pub level: f64,
    #[serde(rename = "limitDistance")]
    pub limit_distance: Option<u8>,
    pub size: u8,
    #[serde(rename = "stopDistance")]
    pub stop_distance: u8,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub working_order_type: WorkingOrderType,
    #[serde(rename = "limitLevel")]
    pub limit_level: Option<f64>,
}

impl Default for CreateWorkingOrderRequest {
    fn default() -> Self {
        let now = Utc::now();
        let dax_utc_close_time = NaiveTime::from_hms(15, 15, 0); // DAX close at 17:30 CET but want to close out WO 15 min before close sommartid CET = Stockholm tid
        let dt_start = NaiveDateTime::new(now.naive_utc().date(), dax_utc_close_time);
        let dt_start_format = dt_start.format("%Y/%m/%d %H:%M:%S").to_string();
        Self {
            time_in_force: "GOOD_TILL_DATE".to_string(),
            good_till_date: dt_start_format,
            deal_reference: "CHANGEME".to_string(),
            epic: "IX.D.DAX.IFMM.IP".to_string(),
            expiry: "-".to_string(),
            direction: Direction::BUY,
            size: 0,
            working_order_type: WorkingOrderType::LIMIT,
            level: 0.,
            guaranteed_stop: false,
            stop_distance: 0,
            limit_distance: None,
            force_open: false, // Is this to be like netting? Dont understand this
            currency_code: "EUR".to_string(),
            limit_level: None,
        }
    }
}

impl CreateWorkingOrderRequest {
    pub fn new(direction: Direction, level: f64, reference: &str, market_info: MarketInfo, target_price: Option<f64>) -> Self {
        let mut val = Self {
            direction,
            level,
            deal_reference: reference.to_string(),
            epic: market_info.epic,
            expiry: market_info.expiry,
            size: market_info.lot_size,
            stop_distance: market_info.stop_distance,
            currency_code: market_info.currency,
            ..CreateWorkingOrderRequest::default()
        };
        if let Some(target) = target_price {
            // If we specify a target
            val.limit_level = Some(target);
        } else {
            // Default target
            val.limit_distance = Some(market_info.stop_distance * 10);
        }
        val
    }
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
    #[serde(rename = "limitLevel")]
    pub limit_level: f64,
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
            size: 0,
            order_type: OrderType::LIMIT,
            level: 0.,
            guaranteed_stop: false,
            stop_distance: 0,
            limit_distance: 0,
            force_open: false,
            currency_code: CurrencyCode::EUR,
            limit_level: 0.,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FetchDataResponse {
    pub prices: Vec<DataPointResponse>,
    #[serde(rename = "instrumentType")]
    pub instrument_type: String,
    pub allowance: Allowance,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataPointResponse {
    #[serde(rename = "closePrice")]
    pub close_price: PricePoint,
    #[serde(rename = "highPrice")]
    pub high_price: PricePoint,
    #[serde(rename = "lowPrice")]
    pub low_price: PricePoint,
    #[serde(rename = "openPrice")]
    pub open_price: PricePoint,
    #[serde(rename = "lastTradedVolume")]
    pub last_traded_volume: usize,
    #[serde(rename = "snapshotTime")]
    pub snapshot_time: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PricePoint {
    pub ask: f64,
    pub bid: f64,
    #[serde(rename = "lastTraded")]
    pub last_traded: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Allowance {
    #[serde(rename = "allowanceExpiry")]
    allowance_expiry: usize,
    #[serde(rename = "remainingAllowance")]
    remaining_allowance: usize,
    #[serde(rename = "totalAllowance")]
    total_allowance: usize,
}

#[cfg(test)]
mod tests {
    use crate::realtime::models::{OpenPositionUpdate, TradeConfirmationUpdate};
    use crate::realtime::notifications::parse_trade_update;
    use crate::FetchDataResponse;

    #[test]
    fn initial_market() {
        let a = r#"{"prices":[{"snapshotTime":"2022/05/18 09:00:00","openPrice":{"bid":14199.1,"ask":14200.5,"lastTraded":null},"closePrice":{"bid":14200.1,"ask":14201.5,"lastTraded":null},"highPrice":{"bid":14203.1,"ask":14204.5,"lastTraded":null},"lowPrice":{"bid":14189.6,"ask":14191.0,"lastTraded":null},"lastTradedVolume":444},{"snapshotTime":"2022/05/18 09:01:00","openPrice":{"bid":14199.6,"ask":14201.0,"lastTraded":null},"closePrice":{"bid":14187.6,"ask":14189.0,"lastTraded":null},"highPrice":{"bid":14201.6,"ask":14203.0,"lastTraded":null},"lowPrice":{"bid":14187.6,"ask":14189.0,"lastTraded":null},"lastTradedVolume":125}],"instrumentType":"INDICES","allowance":{"remainingAllowance":9987,"totalAllowance":10000,"allowanceExpiry":593999}}"#;
        let r = serde_json::from_str::<FetchDataResponse>(a).unwrap();
        let a = 3;
    }
}
