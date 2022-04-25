use bfg_core::bfg_service_impl::BfgServiceImpl;
use bfg_core::domain::{State, SystemValues};
use ig_brokerage_adapter::ig_brokerage_impl::IgBrokerageApi;
use crate::warp_bfg_controller::{init_routes};

mod warp_bfg_controller;

#[tokio::main]
async fn main() {
    // Read configuration
    // Instantiate ig_brokerage_api
    let brokerage = IgBrokerageApi::new();
    let service = BfgServiceImpl {brokerage, state: State::new(SystemValues::new(4, 2))};
    // Get the controller to use from warf_bfg_controller
    init_routes(service).run(([127, 0, 0, 1], 3030)).await;
}

//TODO should have rust integration tests for this file check book how to do that.

/// Questions
/// How are move handled in patter matching, looks like the value is moved? A(a) => .. a is moved?
/// How to put service into handlers and have some parts mutable?