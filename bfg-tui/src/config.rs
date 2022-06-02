use std::env::home_dir;
use std::str::FromStr;
use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, TimeZone, Utc};
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
    pub utc_open_time: String,
    pub utc_close_time: String,
    pub non_trading_days: Vec<String>,
    pub bars_in_opening_range: u8,
}

#[derive(Debug, Clone)]
pub struct BfgConfig {
    pub connection_details: ConnectionDetails,
    pub epics: Vec<MarketInfo>,
}

impl BfgConfig {
    /// Create configuration objects for the markets that has trading hours today.
    /// non_trading_days will be used to filter out configurations.
    pub async fn new() -> Self {
        let mut path = home_dir().expect("always have a home");
        path.push("bfg/demo/config.json");
        let data = fs::read_to_string(path.as_path()).await.expect("Unable to read file");
        let json: InternalConfig = serde_json::from_str(&data)
            .expect("JSON is not the correct format");
        Self {
            connection_details: json.connection_details,
            epics: json.epics.iter().cloned().filter_map(|v| v.try_into().ok()).collect(),
        }
    }
}

impl TryFrom<InternalMarketInfo> for MarketInfo {
    type Error = ();

    fn try_from(value: InternalMarketInfo) -> Result<Self, Self::Error> {
        let today = Utc::today();
        let is_non_trading_day = value.non_trading_days.iter()
            .map(|i| NaiveDate::from_str(i).expect("Failed to parse non_trading_days"))
            .map(|d| Utc::from_utc_date(&Utc, &d))
            .any(|non_trading_day| non_trading_day.eq(&today));
        if is_non_trading_day {
            Err(())
        } else {
            Ok(
                MarketInfo {
                    epic: value.epic,
                    expiry: value.expiry,
                    currency: value.currency,
                    min_tradable_opening_range: value.min_tradable_opening_range,
                    lot_size: value.lot_size,
                    utc_open_time: create_utc_from_time(&value.utc_open_time),
                    utc_close_time: create_utc_from_time(&value.utc_close_time),
                    bars_in_opening_range: value.bars_in_opening_range,
                }
            )
        }
    }
}

fn create_utc_from_time(time: &str) -> DateTime<Utc> {
    let naive_time = NaiveTime::from_str(time).expect("Failed to parse time");
    Utc::now().with_hour(naive_time.hour()).unwrap().with_minute(naive_time.minute()).unwrap().with_second(naive_time.second()).unwrap()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, Utc};

    #[test]
    fn read() {
        let date = NaiveDate::from_str("2022-6-2").unwrap();
        println!("{}", date.to_string());
    }
}