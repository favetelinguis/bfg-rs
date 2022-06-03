use crate::realtime::models::{AccountUpdate, MarketState, OpenPositionUpdate, TradeConfirmationUpdate, WorkingOrderUpdate};
use std::borrow::BorrowMut;
use std::ops::{Add, Sub};
use std::str::FromStr;
use chrono::{DateTime, Duration, Utc};
use log::{info, warn};
use bfg_core::models::get_reference_from_id;
use crate::MarketUpdate;

type MarketState2 = (
    Option<f64>,
    Option<f64>,
    Option<usize>,
    Option<MarketState>,
    Option<String>,
);

type AccountState = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

/// We need to keep the reference unique between markets that is why we append the epic when creating the order.
/// Then we remove it here, we also need to shrink the size since there is a max size in ig api of 30chars that is why we encode the
/// reference as a usize
pub fn parse_trade_update(
    msg: &str,
) -> (
    Option<TradeConfirmationUpdate>,
    Option<OpenPositionUpdate>,
    Option<WorkingOrderUpdate>,
) {
    let parts: Vec<&str> = msg.trim().split('|').collect();

    let conf = if parts[0].starts_with('{') {
        if let Some(mut confirmation) = serde_json::from_str::<TradeConfirmationUpdate>(parts[0]).ok() {
            if !is_old_message(confirmation.date.clone()) {
                let ref_id = confirmation.deal_reference.chars().next().unwrap();
                if let Some(ref_id) = ref_id.to_digit(10) {
                    confirmation.deal_reference = get_reference_from_id(ref_id);
                    Some(confirmation)
                } else {
                    None }
            } else {
                warn!("Skipping CONFS due to old message: {}", parts[0]);
                None }
        } else { None }
    } else {
        None
    };
    let opu = if parts[1].starts_with('{') {
        if let Some(mut confirmation) = serde_json::from_str::<OpenPositionUpdate>(parts[1]).ok() {
            let ref_id = confirmation.deal_reference.chars().next().unwrap();
            let ref_id = ref_id.to_digit(10).unwrap();
            confirmation.deal_reference = get_reference_from_id(ref_id);
            Some(confirmation)
        } else { None }

    } else {
        None
    };
    let wou = if parts[2].starts_with('{') {
        if let Some(mut confirmation) = serde_json::from_str::<WorkingOrderUpdate>(parts[2]).ok() {
            let ref_id = confirmation.deal_reference.chars().next().unwrap();
            let ref_id = ref_id.to_digit(10).unwrap();
            confirmation.deal_reference = get_reference_from_id(ref_id);
            Some(confirmation)
        } else { None }

    } else {
        None
    };

    (conf, opu, wou)
}

/// Check if now minus 10 seconds is before conf time
fn is_old_message(conf_time: String) -> bool {
    // Remove milliseconds
    let conf_time = &String::from("2022-06-02T18:47:43.065")[..19];
    let conf_time: DateTime<Utc> = DateTime::parse_from_rfc3339(format!("{}Z", conf_time).as_str()).unwrap().with_timezone(&Utc);
    conf_time.add(Duration::seconds(10)) >= Utc::now()
}

pub fn parse_account_update(msg: &str, account: String) -> AccountUpdate {
    let mut prev: AccountState = (
        None, None, None, None, None, None, None, None, None, None, None, None,
    );
    let parts: Vec<&str> = msg.trim().split('|').collect();
    let mut indexer: i32 = -1;
    for p in parts {
        if p.starts_with('^') {
            let end: i32 = p.chars().nth(1).unwrap().to_string().parse().unwrap();
            *indexer.borrow_mut() += end
        } else {
            *indexer.borrow_mut() += 1;
            match p {
                "" => {}
                "#" | "$" => match indexer {
                    0 => prev.0 = None,
                    1 => prev.1 = None,
                    2 => prev.2 = None,
                    3 => prev.3 = None,
                    4 => prev.4 = None,
                    5 => prev.5 = None,
                    6 => prev.6 = None,
                    7 => prev.7 = None,
                    8 => prev.8 = None,
                    9 => prev.9 = None,
                    10 => prev.10 = None,
                    11 => prev.11 = None,
                    _ => unreachable!(),
                },
                _ => match indexer {
                    0 => prev.0 = Some(p.parse().unwrap()),
                    1 => prev.1 = Some(p.parse().unwrap()),
                    2 => prev.2 = Some(p.parse().unwrap()),
                    3 => prev.3 = Some(p.parse().unwrap()),
                    4 => prev.4 = Some(p.parse().unwrap()),
                    5 => prev.5 = Some(p.parse().unwrap()),
                    6 => prev.6 = Some(p.parse().unwrap()),
                    7 => prev.7 = Some(p.parse().unwrap()),
                    8 => prev.8 = Some(p.parse().unwrap()),
                    9 => prev.9 = Some(p.parse().unwrap()),
                    10 => prev.10 = Some(p.parse().unwrap()),
                    11 => prev.11 = Some(p.parse().unwrap()),
                    _ => unreachable!(),
                },
            }
        }
    }
    AccountUpdate {
        account,
        pnl: prev.0,
        deposit: prev.1,
        available_cash: prev.2,
        pnl_lr: prev.3,
        pnl_nlr: prev.4,
        funds: prev.5,
        margin: prev.6,
        margin_lr: prev.7,
        margin_nlr: prev.8,
        available_to_deal: prev.9,
        equity: prev.10,
        equity_used: prev.11,
    }
}

pub fn parse_market_update(msg: &str, epic: String) -> MarketUpdate {
    //"BID" "OFFER" "MARKET_DELAY" "MARKET_STATE" "UPDATE_TIME"
    let mut prev: MarketState2 = (None, None, None, None, None);
    let parts: Vec<&str> = msg.trim().split('|').collect();
    let mut indexer: i32 = -1;
    for p in parts {
        if p.starts_with('^') {
            let end: i32 = p.chars().nth(1).unwrap().to_string().parse().unwrap();
            *indexer.borrow_mut() += end
        } else {
            *indexer.borrow_mut() += 1;
            // Check p can be "" # $
            match p {
                "" => {}
                "#" | "$" => match indexer {
                    0 => prev.0 = None,
                    1 => prev.1 = None,
                    2 => prev.2 = None,
                    3 => prev.3 = None,
                    4 => prev.4 = None,
                    _ => unreachable!(),
                },
                _ => match indexer {
                    0 => prev.0 = Some(p.parse().unwrap()),
                    1 => prev.1 = Some(p.parse().unwrap()),
                    2 => prev.2 = Some(p.parse().unwrap()),
                    3 => prev.3 = Some(FromStr::from_str(p).unwrap()),
                    4 => prev.4 = Some(p.to_string()),
                    _ => unreachable!(),
                },
            }
        }
    }
    MarketUpdate {
        epic,
        bid: prev.0,
        offer: prev.1,
        market_delay: prev.2,
        market_state: prev.3.clone(),
        update_time: prev.4,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::{DateTime, Utc};
    use serde_json::from_str;
    use crate::realtime::models::{OpenPositionUpdate, TradeConfirmationUpdate};
    use crate::realtime::notifications::{is_old_message, parse_trade_update};

    #[test]
    fn initial_market() {
        // Rejected WO
        let raw1 = r#"CONFIRMS {"direction":"SELL","epic":"IX.D.DAX.IFM1.IP","stopLevel":null,"│
│         limitLevel":null,"dealReference":"Z32MH7P84M4TYPT","dealId":"DIAAAAJFCQH5VAQ","limitDistance":null,"stopDistance":null,"expiry":null,"affectedDeals":[],"dealStatus":"REJECTED","gu│
│         aranteedStop":false,"trailingStop":false,"level":null,"reason":"ATTACHED_ORDER_LEVEL_ERROR","status":null,"size":null,"profit":null,"profitCurrency":null,"date":"2022-05-20T10:09:│
│         07.865","channel":"PublicRestOTC"}"#;

        // Open WO
        let raw1 = r#"CONFIRMS {"direction":"SELL","epic":"IX.D.DAX.IFM1.IP","stopLevel":null,"│
│         limitLevel":null,"dealReference":"9YQQR7MPUKUTYPH","dealId":"DIAAAAJFCSMG8AJ","limitDistance":null,"stopDistance":null,"expiry":"-","affectedDeals":[{"dealId":"DIAAAAJFCSMG8AJ","s│
│         tatus":"OPENED"}],"dealStatus":"ACCEPTED","guaranteedStop":false,"trailingStop":false,"level":14400,"reason":"SUCCESS","status":"OPEN","size":1,"profit":null,"profitCurrency":null│
│         ,"date":"2022-05-20T10:14:27.134","channel":"PublicRestOTC"}"#;
        let raw1 = r#"OPU {"dealReference":"9YQQR7MPUKUTYPH","dealId":"DIAAAAJFCSMG8AJ","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"OPEN",│
│         "dealStatus":"ACCEPTED","level":14400,"size":1,"timestamp":"2022-05-20T10:14:27.000","channel":"PublicRestOTC","expiry":"-","currency":"EUR","stopDistance":null,"limitDistance":nu│
│         ll,"guaranteedStop":false,"orderType":"LIMIT","timeInForce":"GOOD_TILL_CANCELLED","goodTillDate":null}"#;

        // Manual close WO
        let raw1 = r#"OPU {"dealReference":"9YQQR7MPUKUTYPH","dealId":"DIAAAAJFCSMG8AJ","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"DELETE│
│         D","dealStatus":"ACCEPTED","level":14400,"size":1,"timestamp":"2022-05-20T10:17:29.841","channel":"PublicRestOTC","expiry":"-","currency":"EUR","stopDistance":null,"limitDistance"│
│         :null,"guaranteedStop":false,"orderType":"LIMIT","timeInForce":"GOOD_TILL_CANCELLED","goodTillDate":null}"#;
        // Manual close position
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"DELETE│
│         D","dealStatus":"ACCEPTED","level":14151,"size":0,"timestamp":"2022-05-20T10:40:17.609","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":14151,│
│         "limitLevel":null,"guaranteedStop":false}"#;

        // Delete WO API
        let raw1 = r#"CONFIRMS {"direction":"SELL","epic":"IX.D.DAX.IFM1.IP","stopLevel":null,"│
│         limitLevel":null,"dealReference":"TZB5W79YXN8TYPT","dealId":"DIAAAAJFCSBZXA6","limitDistance":10,"stopDistance":10,"expiry":"-","affectedDeals":[{"dealId":"DIAAAAJFCSBZXA6","statu│
│         s":"DELETED"}],"dealStatus":"ACCEPTED","guaranteedStop":false,"trailingStop":false,"level":14400,"reason":"SUCCESS","status":"DELETED","size":1,"profit":null,"profitCurrency":null│
│         ,"date":"2022-05-20T10:19:47.837","channel":"PublicRestOTC"}"#;
        let raw1 = r#"OPU {"dealReference":"TZB5W79YXN8TYPT","dealId":"DIAAAAJFCSBZXA6","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"DELETE│
│         D","dealStatus":"ACCEPTED","level":14400,"size":1,"timestamp":"2022-05-20T10:19:47.829","channel":"PublicRestOTC","expiry":"-","currency":"EUR","stopDistance":10,"limitDistance":1│
│         0,"guaranteedStop":false,"orderType":"LIMIT","timeInForce":"GOOD_TILL_CANCELLED","goodTillDate":null}"#;

        // WO gets triggered to position
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"DELETE│
│         D","dealStatus":"ACCEPTED","level":14147,"size":1,"timestamp":"2022-05-20T10:31:00.265","channel":"PublicRestOTC","expiry":"-","currency":"EUR","stopDistance":null,"limitDistance"│
│         :null,"guaranteedStop":false,"orderType":"LIMIT","timeInForce":"GOOD_TILL_CANCELLED","goodTillDate":null}"#;
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"OPEN",│
│         "dealStatus":"ACCEPTED","level":14147,"size":1,"timestamp":"2022-05-20T10:31:00.265","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":null,"lim│
│         itLevel":null,"guaranteedStop":false}"#;

        // WO gets updated with floating stop
        let raw1 = r#"CONFIRMS {"direction":"SELL","epic":"IX.D.DAX.IFM1.IP","stopLevel":14155,│
│         "limitLevel":null,"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","limitDistance":null,"stopDistance":null,"expiry":"-","affectedDeals":[{"dealId":"DIAAAAJFCTRTYBB","│
│         status":"AMENDED"}],"dealStatus":"ACCEPTED","guaranteedStop":false,"trailingStop":true,"level":14147.0000,"reason":"SUCCESS","status":"AMENDED","size":1,"profit":null,"profitCurre│
│         ncy":null,"date":"2022-05-20T10:35:03.473","channel":"PublicRestOTC"}"#;
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"UPDATE│
│         D","dealStatus":"ACCEPTED","level":14147.0000,"size":1,"timestamp":"2022-05-20T10:35:03.466","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":1│
│         4155,"limitLevel":null,"guaranteedStop":false}"#;

        // 3 st floating stop updates
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"UPDATE│
│         D","dealStatus":"ACCEPTED","level":14147.0000,"size":1,"timestamp":"2022-05-20T10:35:16.063","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":1│
│         4154,"limitLevel":null,"guaranteedStop":false}"#;
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"UPDATE│
│         D","dealStatus":"ACCEPTED","level":14147.0000,"size":1,"timestamp":"2022-05-20T10:35:23.170","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":1│
│         4153,"limitLevel":null,"guaranteedStop":false}"#;
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"UPDATE│
│         D","dealStatus":"ACCEPTED","level":14147.0000,"size":1,"timestamp":"2022-05-20T10:36:24.131","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":1│
│         4152,"limitLevel":null,"guaranteedStop":false}"#;

        // Floating stop get hit
        let raw1 = r#"OPU {"dealReference":"YYW6WTG7R7UTYPT","dealId":"DIAAAAJFCTRTYBB","direction":"SELL","epic":"IX.D.DAX.IFM1.IP","status":"DELETE│
│         D","dealStatus":"ACCEPTED","level":14151,"size":0,"timestamp":"2022-05-20T10:40:17.609","channel":"OSAutoStopFill","dealIdOrigin":"DIAAAAJFCTRTYBB","expiry":"-","stopLevel":14151,│
│         "limitLevel":null,"guaranteedStop":false}"#;

        let res1 = parse_trade_update("{\"direction\":\"BUY\",\"epic\":\"IX.D.DAX.IFMM.IP\",\"stopLevel\":14192.3,\"limitLevel\":14202.3,\"dealReference\":\"APPPA\",\"dealId\":\"DIAAAAJEBA9SKAW\",\"limitDistance\":null,\"stopDistance\":null,\"expiry\":\"-\",\"affectedDeals\":[{\"dealId\":\"DIAAAAJEBA9SKAW\",\"status\":\"OPENED\"}],\"dealStatus\":\"ACCEPTED\",\"guaranteedStop\":false,\"trailingStop\":false,\"level\":14197.3,\"reason\":\"SUCCESS\",\"status\":\"OPEN\",\"size\":1,\"profit\":null,\"profitCurrency\":null,\"date\":\"2022-05-17T13:43:18.425\",\"channel\":\"PublicRestOTC\"}|");

        let res2 = parse_trade_update("#|{\"dealReference\":\"APPPA\",\"dealId\":\"DIAAAAJEBA9SKAW\",\"direction\":\"BUY\",\"epic\":\"IX.D.DAX.IFMM.IP\",\"status\":\"OPEN\",\"dealStatus\":\"ACCEPTED\",\"level\":14197.3,\"size\":1,\"timestamp\":\"2022-05-17T13:43:18.414\",\"channel\":\"PublicRestOTC\",\"dealIdOrigin\":\"DIAAAAJEBA9SKAW\",\"expiry\":\"-\",\"stopLevel\":14192.3,\"limitLevel\":14202.3,\"guaranteedStop\":false}");

        let a = 23;
    }

    #[test]
    fn bla() {
        let stra = &String::from("2022-06-02T18:47:43.065")[..19];
        println!("RAW {}", stra);
        let utc: DateTime<Utc> = DateTime::parse_from_rfc3339(format!("{}Z", stra).as_str()).unwrap().with_timezone(&Utc);
        println!("{}", utc.to_string());
        let old = is_old_message("2022-06-02T19:39:41.987".to_string());
        println!("This is an old message {}", old)
    }
}
