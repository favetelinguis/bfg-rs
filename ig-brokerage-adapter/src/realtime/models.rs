use crate::BrokerageError;
use bfg_core::models::{BfgTradeStatus, MarketUpdate, TradeConfirmation};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TradeConfirmationUpdate {
    pub direction: Direction,
    pub epic: String,
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
    #[serde(rename = "stopLevel")]
    pub stop_level: Option<f64>,
    #[serde(rename = "limitLevel")]
    pub limit_level: Option<f64>,
    #[serde(rename = "dealId")]
    pub deal_id: String,
    pub expiry: Option<String>,
    #[serde(rename = "affectedDeals")]
    pub affected_deals: Vec<AffectedDeals>,
    #[serde(rename = "dealStatus")]
    pub deal_status: DealStatus,
    pub level: Option<f64>,
    pub reason: String,
    pub status: Option<PositionStatus>,
    pub size: Option<u8>,
    pub profit: Option<f64>,
    #[serde(rename = "profitCurrency")]
    pub profit_currency: Option<String>,
    pub date: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AffectedDeals {
    #[serde(rename = "dealId")]
    pub deal_id: String,
    pub status: AffectedDealStatus,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum DealStatus {
    ACCEPTED,
    REJECTED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PositionStatus {
    AMENDED,
    CLOSED,
    DELETED,
    OPEN,
    PARTIALLY_CLOSED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AffectedDealStatus {
    AMENDED,
    DELETED,
    FULLY_CLOSED,
    OPENED,
    PARTIALLY_CLOSED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WorkingOrderUpdate {
    pub direction: String,
    #[serde(rename = "limitDistance")]
    pub limit_distance: u8,
    #[serde(rename = "dealId")]
    pub deal_id: String,
    #[serde(rename = "stopDistance")]
    pub stop_distance: u8,
    pub expiry: String,
    pub timestamp: String,
    pub size: usize,
    pub status: OpuStatus,
    pub epic: String,
    pub level: f64,
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
    #[serde(rename = "dealStatus")]
    pub deal_status: DealStatus,
    pub currency: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "goodTillDate")]
    pub good_till_date: String,
    pub channel: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenPositionUpdate {
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
    #[serde(rename = "dealId")]
    pub deal_id: String,
    pub direction: String,
    pub epic: String,
    pub status: OpuStatus,
    #[serde(rename = "dealStatus")]
    pub deal_status: DealStatus,
    pub level: f64,
    pub size: usize,
    pub timestamp: String,
    pub channel: String,
    #[serde(rename = "dealIdOrigin")]
    pub deal_id_origin: String,
    pub expiry: String,
    #[serde(rename = "stopLevel")]
    pub stop_level: Option<f64>,
    #[serde(rename = "limitLevel")]
    pub limit_level: Option<f64>,
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub enum OpuStatus {
    OPEN,
    UPDATED,
    DELETED,
}

impl From<PositionStatus> for bfg_core::models::ConfirmsStatus {
    fn from(input: PositionStatus) -> Self {
        match input {
            PositionStatus::OPEN => bfg_core::models::ConfirmsStatus::OPEN,
            PositionStatus::CLOSED => bfg_core::models::ConfirmsStatus::CLOSED,
            PositionStatus::PARTIALLY_CLOSED => bfg_core::models::ConfirmsStatus::PARTIALLY_CLOSED,
            PositionStatus::AMENDED => bfg_core::models::ConfirmsStatus::AMENDED,
            PositionStatus::DELETED => bfg_core::models::ConfirmsStatus::DELETED,
        }
    }
}

impl From<OpuStatus> for BfgTradeStatus {
    fn from(input: OpuStatus) -> Self {
        match input {
            OpuStatus::OPEN => BfgTradeStatus::OPEN,
            OpuStatus::DELETED => BfgTradeStatus::DELETED,
            OpuStatus::UPDATED => BfgTradeStatus::UPDATED,
        }
    }
}

impl From<DealStatus> for bfg_core::models::DealStatus {
    fn from(input: DealStatus) -> Self {
        match input {
            DealStatus::ACCEPTED => bfg_core::models::DealStatus::ACCEPTED,
            DealStatus::REJECTED => bfg_core::models::DealStatus::REJECTED,
        }
    }
}

#[derive(Debug)]
pub enum Mode {
    Merge,
    Distinct,
}

#[derive(Debug)]
pub enum TlcpRequest {
    CreateSession {
        user: String,
        client_token: String,
        account_token: String,
    },
    BindSession {
        session_id: String,
    },
    Subscribe {
        session_id: String,
        req_id: usize,
        sub_id: usize,
        item: String,
        fields: Vec<String>,
        mode: Mode,
    },
    UnSubscribe {
        session_id: String,
        req_id: usize,
        sub_id: usize,
    },
    Disconnect {
        session_id: String,
        req_id: usize,
    },
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Distinct => write!(f, "DISTINCT"),
            Mode::Merge => write!(f, "MERGE"),
        }
    }
}

impl Display for TlcpRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TlcpRequest::CreateSession {
                user,
                client_token,
                account_token,
            } => {
                let password = format!("CST-{client_token}|XST-{account_token}");
                let payload = format!(
                    "LS_user={user}&LS_password={password}&LS_cid=mgQkwtwdysogQz2BJ4Ji%20kOj2Bg"
                );
                write!(f, "create_session\r\n{payload}")
            }
            TlcpRequest::BindSession { session_id } => {
                let payload = format!("LS_session={session_id}");
                write!(f, "bind_session\r\n{payload}")
            }
            TlcpRequest::Subscribe {
                session_id,
                req_id,
                sub_id,
                item,
                fields,
                mode,
            } => {
                let schema = fields.join(" ");
                let payload = format!("LS_session={session_id}&LS_reqId={req_id}&LS_subId={sub_id}&LS_op=add&LS_mode={mode}&LS_group={item}&LS_schema={schema}&LS_snapshot=true");
                write!(f, "control\r\n{payload}")
            }
            TlcpRequest::UnSubscribe {
                session_id,
                req_id,
                sub_id,
            } => {
                let payload = format!(
                    "LS_session={session_id}&LS_reqId={req_id}&LS_subId={sub_id}&LS_op=delete"
                );
                write!(f, "control\r\n{payload}")
            }
            TlcpRequest::Disconnect { session_id, req_id } => {
                let payload = format!("LS_session={session_id}&LS_reqId={req_id}&LS_op=destroy");
                write!(f, "control\r\n{payload}")
            }
        }
    }
}

impl From<TlcpRequest> for Message {
    fn from(input: TlcpRequest) -> Self {
        Message::Text(format!("{}", input))
    }
}

#[derive(Debug)]
pub enum TlcpResponse {
    // Session Creation or Binding Response
    CONOK {
        session_id: String,
        request_limit: usize,
        keep_alive: usize,
        control_link: String,
    },
    CONERR {
        error_code: u8,
        error_message: String,
    },
    END {
        cause_code: u8,
        cause_message: String,
    },
    // --
    SUBOK {
        subscription_id: u8,
        num_items: u8,
        num_fields: u8,
    },
    UNSUB {
        subscription_id: u8,
    },
    U {
        subscription_id: u8,
        item: u8,
        fields_values: String,
    }, // Check page 45 for how to decode message
    // Control Responses
    REQOK {
        request_id: usize,
    },
    REQERR {
        request_id: usize,
        error_code: u8,
        error_message: String,
    },
    ERROR {
        error_code: u8,
        error_message: String,
    },
    LOOP {
        expected_delay: usize,
    },
    SYNC {
        seconds_since_initial_header: usize,
    },
    PROBE,
    UNKNOWN(String),
}

impl FromStr for TlcpResponse {
    type Err = BrokerageError;

    fn from_str(m: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = m.trim().split(',').collect();
        let res = match parts[0] {
            "CONOK" => TlcpResponse::CONOK {
                session_id: parts[1].to_string(),
                request_limit: parts[2].parse().unwrap(),
                keep_alive: parts[3].parse().unwrap(),
                control_link: parts[4].to_string(),
            },
            "CONERR" => TlcpResponse::CONERR {
                error_code: parts[1].parse().unwrap(),
                error_message: parts[2].to_string(),
            },
            "REQERR" => TlcpResponse::REQERR {
                request_id: parts[1].parse().unwrap(),
                error_code: parts[2].parse().unwrap(),
                error_message: parts[3].to_string(),
            },
            "END" => TlcpResponse::END {
                cause_code: parts[1].parse().unwrap(),
                cause_message: parts[2].to_string(),
            },
            "U" => TlcpResponse::U {
                subscription_id: parts[1].parse().unwrap(),
                item: parts[2].parse().unwrap(),
                fields_values: m[6..].to_string(), // To be able to handle , in JSON object for trade subscription
            },
            "SYNC" => TlcpResponse::SYNC {
                seconds_since_initial_header: parts[1].parse().unwrap(),
            },
            "LOOP" => TlcpResponse::LOOP {
                expected_delay: parts[1].parse().unwrap(),
            },
            "PROBE" => TlcpResponse::PROBE,
            s => TlcpResponse::UNKNOWN(String::from(s)),
        };
        Ok(res)
    }
}

#[derive(Debug)]
pub struct RestDetails {
    pub xst: String,
    pub cst: String,
    pub url: String,
    pub account: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_market() {
        let msg = r#"U,3,1,|{"dealReference":"2DZZ73A1G2YQDSS58","dealId":"DIAAAAJC2PQZ6A9","direction":"SELL","epic":"IX.D.DAX.IFMM.IP","status":"DELETED","dealStatus":"ACCEPTED","level":13951.5,"size":0,"timestamp":"2022-05-13T12:55:48.761","channel":"WTP","dealIdOrigin":"DIAAAAJC2PQZ6A9","expiry":"-","stopLevel":null,"limitLevel":null,"guaranteedStop":false}|"#;
        let result = TlcpResponse::from_str(msg);
        let a = 1;
    }
}
