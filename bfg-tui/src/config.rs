use std::env::home_dir;
use std::str::FromStr;
use chrono::{NaiveDate, NaiveTime};
use bfg_ig::models::{ConnectionDetails, MarketInfo};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InternalConfig {
    pub connection_details: ConnectionDetails,
    pub epics: Vec<InternalMarketInfo>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalMarketInfo {
    pub epic: String,
    pub expiry: String,
    pub currency: String,
    pub min_tradable_opening_range: f64,
    pub lot_size: u8,
    pub open_time: String,
    pub close_time: String,
    pub start_fetch_data: String,
    pub utc_close_working_order: String,
    pub non_trading_days: Vec<String>,
    pub bars_in_opening_range: u8,
}

#[derive(Debug, Clone)]
pub struct BfgConfig {
    pub connection_details: ConnectionDetails,
    pub epics: Vec<MarketInfo>,
}

impl BfgConfig {
    pub async fn new() -> Self {
        let mut path = home_dir().expect("always have a home");
        path.push("bfg/demo/config.json");
        let data = fs::read_to_string(path.as_path()).await.expect("Unable to read file");
        let json: InternalConfig = serde_json::from_str(&data)
            .expect("JSON is not the correct format");
        Self {
            connection_details: json.connection_details,
            epics: json.epics.iter().cloned().map(|v| v.into()).collect(),
        }
    }
}

impl From<InternalMarketInfo> for MarketInfo {
    fn from(val: InternalMarketInfo) -> Self {
        MarketInfo {
            epic: val.epic,
            expiry: val.expiry,
            currency: val.currency,
            min_tradable_opening_range: val.min_tradable_opening_range,
            lot_size: val.lot_size,
            open_time: NaiveTime::from_str(val.open_time.as_str()).expect("Failed to parse open_time"),
            close_time: NaiveTime::from_str(val.close_time.as_str()).expect("Failed to parse close_time"),
            start_fetch_data: NaiveTime::from_str(val.start_fetch_data.as_str()).expect("Failed to parse start_fetch_data"),
            utc_close_working_order: NaiveTime::from_str(val.utc_close_working_order.as_str()).expect("Failed to parse utc_close_working_order"),
            non_trading_days: val.non_trading_days.iter().map(|i| NaiveDate::from_str(i).expect("Failed to parse non_trading_days")).collect(),
            bars_in_opening_range: val.bars_in_opening_range,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BfgConfig;

    #[tokio::test]
    async fn read() {
        let conf = BfgConfig::new().await;
    }
}