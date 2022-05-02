use std::collections::HashMap;
use std::env;
use reqwest::blocking::{Client, Request, Response};
use bfg_core::ports::{BrokerageApi, Or, OrderDetails};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;
use bfg_core::errors::BrokerageError;
use crate::errors::ApiLayerError;
use crate::ig_brokerage_impl::ConnectionState::HasSession;
use crate::models::{ApiResponse, CreateSessionRequest, CreateSessionResponse, GetMarketResponse};

#[derive(Clone)]
pub struct ConnectionDetails {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub account: String,
    pub base_url: String,
    pub epic: String
}

impl ConnectionDetails {
    pub fn fromEnv() -> Self {
        Self {
            username: env::var("IG_USER").expect("IG_USER not set"),
            password: env::var("IG_PASSWORD").expect("IG_PASSWORD not set"),
            api_key: env::var("IG_APIKEY").expect("IG_API_KEY not set"),
            base_url: env::var("IG_BASEURL").expect("IG_BASEURL not set"),
            account: env::var("IG_ACCOUNT").expect("IG_ACCOUNT not set"),
            epic: env::var("EPIC").expect("EPIC not set"),
        }
    }
}

#[derive(Clone)]
enum ConnectionState {
    NoSession,
    HasSession(CreateSessionResponse),
}

#[derive(Clone)]
pub struct IgBrokerageApi {
    connection_details: ConnectionDetails,
    pub http_client: reqwest::blocking::Client,
    state: ConnectionState,
}

impl BrokerageApi for IgBrokerageApi {
    fn get_or(&mut self) -> Option<Or> {
        // Build request and call do_request
        todo!()
    }

    fn place_order(&mut self, order: OrderDetails) {
        // Build request and call do_request
        todo!()
    }

    fn get_market_details(&mut self) -> Result<GetMarketResponse, BrokerageError> {
        match self.state.clone() {
            ConnectionState::NoSession => {
                println!("NOSESSION");
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
                headers.insert(CONTENT_TYPE, "application/json; charset=UTF-8".parse().unwrap());
                headers.insert("X-IG-API-KEY", self.connection_details.api_key.parse().unwrap());
                headers.insert("IG-ACCOUNT-ID", self.connection_details.account.parse().unwrap());
                headers.insert("Version", "3".parse().unwrap());
                let res = self.http_client
                    .post(format!("{}{}", self.connection_details.base_url, "session"))
                    .headers(headers)
                    .json::<CreateSessionRequest>(&CreateSessionRequest {identifier: self.connection_details.username.clone(), password: self.connection_details.password.clone()})
                    .send()
                    .map_err(|e| BrokerageError::CoreBrokerageError)?;
                if !res.status().is_success() {
                    let status = res.status().as_u16();
                    let message = res.text().unwrap();//.json::<ApiResponse>().unwrap();

                    let err = ApiLayerError {
                        status, message
                    };
                    // if status < 500 {
                    //     return Client error
                    // } else {
                    //     return Server error actix
                    // }
                    println!("{:?}", err);
                    return Err(BrokerageError::CoreBrokerageError)
                }

                let res = res.json::<CreateSessionResponse>()
                    .map_err(|e| BrokerageError::CoreBrokerageError)?;

                println!("{:?}", res);
                self.state = HasSession(res);
                self.get_market_details()
            },
            ConnectionState::HasSession(session) => {
                println!("HASSESSION");
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
                headers.insert(CONTENT_TYPE, "application/json; charset=UTF-8".parse().unwrap());
                headers.insert("X-IG-API-KEY", self.connection_details.api_key.parse().unwrap());
                headers.insert("IG-ACCOUNT-ID", self.connection_details.account.parse().unwrap());

                headers.insert("Version", "2".parse().unwrap());
                let res = self.http_client
                    .get(format!("{}{}/{}", self.connection_details.base_url, "markets", self.connection_details.epic))
                    .bearer_auth(session.oauth_token.access_token)
                    .headers(headers)
                    .send()
                    .map_err(|e| BrokerageError::CoreBrokerageError)?;
                println!("NOSESSION3");

                if !res.status().is_success() {
                    let status = res.status().as_u16();
                    let message = res.text().unwrap();//.json::<ApiResponse>().unwrap();

                    println!("NOSESSION3");
                    let err = ApiLayerError {
                        status, message
                    };
                    // if status < 500 {
                    //     return Client error
                    // } else {
                    //     return Server error actix
                    // }
                    println!("{:?}", err);
                    return Err(BrokerageError::CoreBrokerageError)
                }
                println!("NOSESSION3");
                res.json::<GetMarketResponse>()
                    .map_err(|e| {
                        println!("{}", e);
                        BrokerageError::CoreBrokerageError
                    })?;
            }
        }
    }
}

impl IgBrokerageApi {
    pub fn new(connection_details: ConnectionDetails) -> IgBrokerageApi {
        IgBrokerageApi {
            connection_details,
            http_client: Client::builder().build().unwrap(),
            state: ConnectionState::NoSession
        }
    }
}

/// This testsuite test agains the testenvironment of IG but its real integration tests
/// Is there a way to exclude there tests from ordinary cargo test?
/// Maybe instead of integration tests there should be an example
#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;

    #[ignore]
    #[test]
    fn test_save_and_check() {
        dotenv().ok();
        let mut sut = IgBrokerageApi::new(ConnectionDetails::fromEnv());
        sut.get_market_details().unwrap()
    }
}
