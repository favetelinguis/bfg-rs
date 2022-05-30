// TODO should probable use reexports more instead of copy paste structs in the structure dont diverge
pub use ig_brokerage_adapter::ConnectionDetails;
pub use bfg_core::decider::MarketInfo;

#[derive(Default, Debug, Clone)]
pub struct MarketView{
    pub epic: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub market_delay: Option<usize>,
    pub market_state: Option<String>,
    pub update_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConnectionInformationView {
    pub stream_status: String,
}

impl Default for ConnectionInformationView {
    fn default() -> Self {
        Self {
            stream_status: String::from("Not Connected"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeResultView {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: String,
    pub exit_time: String,
    pub exit_level: f64,
    pub reference: String,
    pub epic: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountView {
    pub account: String,
    pub pnl: Option<f64>,
    pub deposit: Option<f64>,
    pub available_cash: Option<f64>,
    pub pnl_lr: Option<f64>,
    pub pnl_nlr: Option<f64>,
    pub funds: Option<f64>,
    pub margin: Option<f64>,
    pub margin_lr: Option<f64>,
    pub margin_nlr: Option<f64>,
    pub available_to_deal: Option<f64>,
    pub equity: Option<f64>,
    pub equity_used: Option<f64>,
}
