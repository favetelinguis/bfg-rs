use bfg_core::bfg_service_impl::BfgServiceImpl;
use bfg_core::domain::State;
use crate::ig_brokerage_api::IgBrokerageApi;
use crate::warp_bfg_controller::{init_routes};

mod ig_brokerage_api;
mod warp_bfg_controller;

#[tokio::main]
async fn main() {
    // Read configuration
    // Instantiate ig_brokerage_api
    let brokerage = IgBrokerageApi::new();
    let service = BfgServiceImpl {brokerage, state: State::Init};
    // Get the controller to use from warf_bfg_controller
    init_routes(service).run(([127, 0, 0, 1], 3030)).await;
}

//TODO should have rust integration tests for this file check book how to do that.
