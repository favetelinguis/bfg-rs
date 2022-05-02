use std::sync::Mutex;
use actix_web::web;
use bfg_core::ports::BfgService;
use crate::handlers::market::{get_market, update_market};

pub fn configure_routes<T: 'static + BfgService>(service: web::Data<Mutex<T>>, cfg: &mut web::ServiceConfig) {
    cfg
        .app_data(service)
        // .service(web::scope("/market"))
        .route("/market", web::get().to(get_market::<T>))
        .route("/market", web::post().to(update_market::<T>));
}
