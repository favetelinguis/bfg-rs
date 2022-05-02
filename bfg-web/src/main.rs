//TODO should have rust integration tests for this file check book how to do that.

/// Questions
/// How are move handled in patter matching, looks like the value is moved? A(a) => .. a is moved?
/// How to put service into handlers and have some parts mutable?
use std::{io, env};
use std::sync::Mutex;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use bfg_core::bfg_service_impl::BfgServiceImpl;
use bfg_core::domain::{State, SystemValues};
use ig_brokerage_adapter::ig_brokerage_impl::{ConnectionDetails, IgBrokerageApi};

mod handlers;
mod models;
mod routes;
mod errors;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let market = env::var("MARKET").expect("MARKET not set");
    // Instantiate ig_brokerage_api
    let brokerage = IgBrokerageApi::new(ConnectionDetails {
        username: env::var("IG_USER").expect("IG_USER not set"),
        password: env::var("IG_PASSWORD").expect("IG_PASSWORD not set"),
        api_key: env::var("IG_APIKEY").expect("IG_API_KEY not set"),
        base_url: env::var("IG_BASEURL").expect("IG_BASEURL not set"),
        account: env::var("IG_ACCOUNT").expect("IG_ACCOUNT not set"),
    });
    let service = web::Data::new(Mutex::new(BfgServiceImpl { brokerage, state: State::new(SystemValues::new(4, 2)) }));

    let app = move || {
        App::new()
            .configure(|cfg| routes::configure_routes(service.clone(), cfg))
    };

    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
