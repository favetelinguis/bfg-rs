use std::borrow::BorrowMut;
use warp::{Filter, Rejection, Reply, Server};
use warp::body::BodyDeserializeError;
use warp::http::StatusCode;
use warp::reject::Reject;
use crate::ports::{BfgService};
use serde::Deserialize;

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameter"),
        }
    }
}

impl Reject for Error {}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

pub fn init_routes<A: BfgServiceImpl>(service: A) {
    let market_event = warp::post()
        .and(warp::path("event/market"))
        .and(warp::path::end())
        .and(service.clone())
        .and(warp::body::json())
        .and_then(handle_market_event);

    let account_event = warp::post()
        .and(warp::path("event/account"))
        .and(warp::path::end())
        .and(service.clone())
        .and(warp::body::json())
        .and_then(handle_account_event);

    let trade_event = warp::post()
        .and(warp::path("event/trade"))
        .and(warp::path::end())
        .and(service.clone())
        .and(warp::body::json())
        .and_then(handle_trade_event);

    let routes = market_event
        .or(account_event)
        .or(trade_event)
        .recover(return_error);

    warp::serve(routes)
}

#[derive(Deserialize, Debug)]
struct IgMarketUpdate {
    bla: usize
}

pub fn handle_market_event<A: BfgServiceImpl>(service: &mut A, body: IgMarketUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    service.publish_market_update_event(body); // TODO should have use trait into or something to convert between types?
    Ok(warp::reply::with_status("Market event handled", StatusCode::OK))
}

#[derive(Deserialize, Debug)]
struct IgAccountUpdate {
    bla: usize
}

pub fn handle_account_event<A: BfgServiceImpl>(service: A, body: IgAccountUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    service.publish_account_update_event(body); // TODO should have use trait into or something to convert between types?
    Ok(warp::reply::with_status("Account event handled", StatusCode::OK))
}

#[derive(Deserialize, Debug)]
struct IgTradeUpdate {
    bla: usize
}

pub fn handle_trade_event<A: BfgServiceImpl>(service: A, body: IgTradeUpdate) -> Result<impl warp::Reply, warp::Rejection> {
    service.publish_trade_update_event(body); // TODO should have use trait into or something to convert between types?
    Ok(warp::reply::with_status("Trade event handled", StatusCode::OK))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controllers() {
        // TODO
        // Use warp built in test tools to test the controllers and the port
        // the BfgService should be mocked here we should not use the real impl
        todo!()
    }
}
