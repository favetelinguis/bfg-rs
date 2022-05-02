use std::sync::Mutex;
use actix_web::{HttpResponse, web};
use bfg_core::bfg_service_impl::BfgServiceImpl;
use bfg_core::ports::{Action, BfgService, BrokerageApi, MarketUpdate, MarketValues};
use ig_brokerage_adapter::ig_brokerage_impl::IgBrokerageApi;
use serde::{Serialize, Deserialize};
use crate::models::market::{MarketDetails, MarketEvent};


pub async fn update_market<T: BfgService>(event: web::Json<MarketEvent>, service: web::Data<Mutex<T>>) -> HttpResponse {
    let market_event:  MarketEvent = event.into();
    service.lock().unwrap().create_session();
    HttpResponse::Ok().json("Market event handled")
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use bfg_core::domain::{State, SystemValues};
    use bfg_core::ports::BfgService;
    use super::*;

    #[actix_rt::test]
    async fn post_market_event_test() {
        let event = web::Json(MarketEvent { high: 44});

        let service = web::Data::new(Mutex::new(BfgMock {}));
        let resp = update_market(event, service).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    struct BfgMock {}

    impl BfgService for BfgMock {
        fn market_details(&self) -> MarketValues {
            todo!()
        }

        fn setup_market(&mut self, market: MarketValues) {
            todo!()
        }

        fn publish_update_event(&mut self, update: Action) {
            ()
        }
    }
}