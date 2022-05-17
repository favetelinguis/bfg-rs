use crate::realtime::models::{OpenPositionUpdate, TradeConfirmationUpdate};
use bfg_core::models::{AccountUpdate, MarketUpdate};
use std::borrow::BorrowMut;

type MarketState = (
    Option<f64>,
    Option<f64>,
    Option<usize>,
    Option<String>,
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

pub fn parse_trade_update(
    msg: &str,
) -> (Option<TradeConfirmationUpdate>, Option<OpenPositionUpdate>) {
    let parts: Vec<&str> = msg.trim().split('|').collect();

    let conf = if parts[0].starts_with('{') {
        serde_json::from_str::<TradeConfirmationUpdate>(parts[0]).ok()
    } else {
        None
    };
    let opu = if parts[1].starts_with('{') {
        serde_json::from_str::<OpenPositionUpdate>(parts[1]).ok()
    } else {
        None
    };

    (conf, opu)
}

pub fn parse_account_update(msg: &str) -> AccountUpdate {
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
        account: None, // Hackish since i never want it updated
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

pub fn parse_market_update(msg: &str) -> MarketUpdate {
    //"BID" "OFFER" "MARKET_DELAY" "MARKET_STATE" "UPDATE_TIME"
    let mut prev: MarketState = (None, None, None, None, None);
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
                    3 => prev.3 = Some(p.to_string()),
                    4 => prev.4 = Some(p.to_string()),
                    _ => unreachable!(),
                },
            }
        }
    }
    MarketUpdate {
        bid: prev.0,
        offer: prev.1,
        market_delay: prev.2,
        market_state: prev.3.clone(),
        update_time: prev.4,
    }
}

#[cfg(test)]
mod tests {
    use crate::realtime::models::{OpenPositionUpdate, TradeConfirmationUpdate};
    use crate::realtime::notifications::parse_trade_update;

    #[test]
    fn initial_market() {
        let res1 = parse_trade_update("{\"direction\":\"BUY\",\"epic\":\"IX.D.DAX.IFMM.IP\",\"stopLevel\":14192.3,\"limitLevel\":14202.3,\"dealReference\":\"APPPA\",\"dealId\":\"DIAAAAJEBA9SKAW\",\"limitDistance\":null,\"stopDistance\":null,\"expiry\":\"-\",\"affectedDeals\":[{\"dealId\":\"DIAAAAJEBA9SKAW\",\"status\":\"OPENED\"}],\"dealStatus\":\"ACCEPTED\",\"guaranteedStop\":false,\"trailingStop\":false,\"level\":14197.3,\"reason\":\"SUCCESS\",\"status\":\"OPEN\",\"size\":1,\"profit\":null,\"profitCurrency\":null,\"date\":\"2022-05-17T13:43:18.425\",\"channel\":\"PublicRestOTC\"}|");

        let res2 = parse_trade_update("#|{\"dealReference\":\"APPPA\",\"dealId\":\"DIAAAAJEBA9SKAW\",\"direction\":\"BUY\",\"epic\":\"IX.D.DAX.IFMM.IP\",\"status\":\"OPEN\",\"dealStatus\":\"ACCEPTED\",\"level\":14197.3,\"size\":1,\"timestamp\":\"2022-05-17T13:43:18.414\",\"channel\":\"PublicRestOTC\",\"dealIdOrigin\":\"DIAAAAJEBA9SKAW\",\"expiry\":\"-\",\"stopLevel\":14192.3,\"limitLevel\":14202.3,\"guaranteedStop\":false}");

        let a = 23;
    }

    #[test]
    fn bla() {

        let a = serde_json::from_str::<TradeConfirmationUpdate>("{\"direction\":\"BUY\",\"epic\":\"IX.D.DAX.IFMM.IP\",\"stopLevel\":14192.3,\"limitLevel\":14202.3,\"dealReference\":\"APPPA\",\"dealId\":\"DIAAAAJEBA9SKAW\",\"limitDistance\":null,\"stopDistance\":null,\"expiry\":\"-\",\"affectedDeals\":[{\"dealId\":\"DIAAAAJEBA9SKAW\",\"status\":\"OPENED\"}],\"dealStatus\":\"ACCEPTED\",\"guaranteedStop\":false,\"trailingStop\":false,\"level\":14197.3,\"reason\":\"SUCCESS\",\"status\":\"OPEN\",\"size\":1,\"profit\":null,\"profitCurrency\":null,\"date\":\"2022-05-17T13:43:18.425\",\"channel\":\"PublicRestOTC\"}").unwrap();
        let b = 3;
    }
}
