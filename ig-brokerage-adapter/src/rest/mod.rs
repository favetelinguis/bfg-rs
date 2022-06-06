use crate::rest::models::{
    AccessTokenResponse, ClosePositionRequest, CreateDeletePositionResponse, CreateSessionRequest,
    CreateSessionRequestV2, CreateSessionResponse, CreateSessionResponseV2,
    CreateWorkingOrderRequest, EditPositionRequest, FetchDataResponse, OpenPositionRequest,
    RefreshTokenRequest,
};
use chrono_tz::Europe::{London, Stockholm};
use crate::{BrokerageError, ConnectionDetails, SessionState};
use bfg_core::models::Direction;
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::ops::{Add, Sub};
use std::sync::Arc;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use log::{error, info, warn};
use tokio::sync::Mutex;
use bfg_core::decider::MarketInfo;

pub mod models;

pub struct NoSession;
pub struct HasSession;

pub struct IgRestClient<State = NoSession> {
    state: PhantomData<State>,
    client: Client,
    session: Arc<Mutex<SessionState>>,
    connection_details: ConnectionDetails,
}

impl IgRestClient<NoSession> {
    pub fn new(session: Arc<Mutex<SessionState>>, connection_details: ConnectionDetails) -> Self {
        Self {
            state: PhantomData,
            session,
            connection_details,
            client: Client::builder().build().unwrap(),
        }
    }

    pub async fn create_session(self) -> Result<IgRestClient<HasSession>, BrokerageError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=UTF-8".parse().unwrap(),
        );
        headers.insert(
            "X-IG-API-KEY",
            self.connection_details.api_key.parse().unwrap(),
        );
        headers.insert("Version", "2".parse().unwrap());
        let res = self
            .client
            .post(format!("{}{}", self.connection_details.base_url, "session"))
            .headers(headers)
            .json(&CreateSessionRequestV2 {
                encrypted_password: false,
                identifier: self.connection_details.username.clone(),
                password: self.connection_details.password.clone(),
            })
            .send()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            error!("Failed creating session with IG {} {}", status, message);
            return Err(BrokerageError(format!("create_session failure with status: {} message: {}", status, message)));
        }

        // xst och cst is good for 12h and will reset on each api call for another 12h
        let xst = res
            .headers()
            .get("x-security-token")
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        let cst = res.headers().get("cst").unwrap().to_str().unwrap().into();

        let res = res
            .json::<CreateSessionResponseV2>()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;

        // Set the shared session after login
        *self.session.lock().await = SessionState {
            xst,
            cst,
            lightstreamer_endpoint: res.lightstreamer_endpoint.clone(),
            account: self.connection_details.account.clone(),
        };

        Ok(IgRestClient::<HasSession> {
            client: self.client,
            session: self.session,
            state: PhantomData,
            connection_details: self.connection_details,
        })
    }
}

impl IgRestClient<HasSession> {
    /// fetch data api expects local time for account wich in my case is stockholm
    pub async fn fetch_data(
        &self,
        epic: &str,
        start: DateTime<Utc>,
        duration: Duration,
    ) -> Result<FetchDataResponse, BrokerageError> {
        let start = start.with_timezone(&Stockholm);
        let dt_start_format = start.format("%Y-%m-%dT%H:%M:%S").to_string();
        // We always substract 1minute since we send number bars wich will always be minimum 1
        // but in reality will give us 2 bars when start 9:00 and end 9:01
        // we need start 9:00 and end 9:00 to only get one bar back
        let dt_end = start.add(duration.sub(Duration::minutes(1)));
        let dt_end_format = dt_end.format("%Y-%m-%dT%H:%M:%S").to_string();
        let SessionState {
            ref xst, ref cst, ..
        } = &*self.session.lock().await;
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=UTF-8".parse().unwrap(),
        );
        headers.insert(
            "X-IG-API-KEY",
            self.connection_details.api_key.parse().unwrap(),
        );
        headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
        headers.insert("CST", cst.parse().unwrap());
        headers.insert("Version", "3".parse().unwrap());
        let resource = format!("prices/{epic}?resolution=MINUTE&from={dt_start_format}&to={dt_end_format}");
        let url = format!("{}{}", self.connection_details.base_url, resource);
        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            return Err(BrokerageError(format!("fetch_data fail with status: {} message: {}", status, message)));
        }

        let res = res
            .json::<FetchDataResponse>()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;

        return Ok(res);
    }

    pub async fn open_working_order(
        &self,
        direction: Direction,
        level: f64,
        deal_reference: &str,
        market_info: MarketInfo,
        target_distance: f64,
        stop_distance: f64,
    ) -> Result<(), BrokerageError> {
        let SessionState { xst, cst, .. } = &*self.session.lock().await;
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=UTF-8".parse().unwrap(),
        );
        headers.insert(
            "X-IG-API-KEY",
            self.connection_details.api_key.parse().unwrap(),
        );
        headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
        headers.insert("CST", cst.parse().unwrap());
        headers.insert("Version", "2".parse().unwrap());
        let request_body =
            CreateWorkingOrderRequest::new(direction.clone().into(), level, deal_reference, market_info, target_distance, stop_distance);
        let res = self
            .client
            .post(format!(
                "{}{}",
                self.connection_details.base_url, "workingorders/otc"
            ))
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            return Err(BrokerageError(format!("open_working_order fail with status: {} message: {}", status, message)));
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;

        return Ok(());
    }

    pub async fn delete_working_order(&self, deal_id: &str) -> Result<(), BrokerageError> {
        let SessionState {
            ref xst, ref cst, ..
        } = &*self.session.lock().await;
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=UTF-8".parse().unwrap(),
        );
        headers.insert(
            "X-IG-API-KEY",
            self.connection_details.api_key.parse().unwrap(),
        );
        headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
        headers.insert("CST", cst.parse().unwrap());
        headers.insert("Version", "2".parse().unwrap());
        headers.insert("_method", "DELETE".parse().unwrap());
        let res = self
            .client
            .post(format!(
                "{}{}/{}",
                self.connection_details.base_url, "workingorders/otc", deal_id
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            warn!(
                "Failed to close workingorder for deal id {} with status {} and with reason: {}",
                deal_id, status, message
            );

            // TODO should not always say ok, only say ok for specific message and code that happen when closing already closed
            return Ok(()); // If i try to close a position that already is closed things crash so i always say ok even if not
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        return Ok(());
    }

    pub async fn edit_position(
        &self,
        deal_id: &str,
        stop_level: f64,
        trailing_stop_distance: f64,
        target_level: f64,
    ) -> Result<(), BrokerageError> {
        let SessionState {
            ref xst, ref cst, ..
        } = &*self.session.lock().await;
        let body = EditPositionRequest::new(stop_level, trailing_stop_distance, target_level);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
        headers.insert(
            CONTENT_TYPE,
            "application/json; charset=UTF-8".parse().unwrap(),
        );
        headers.insert(
            "X-IG-API-KEY",
            self.connection_details.api_key.parse().unwrap(),
        );
        headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
        headers.insert("CST", cst.parse().unwrap());
        headers.insert("Version", "2".parse().unwrap());
        let res = self
            .client
            .put(format!(
                "{}{}/{}",
                self.connection_details.base_url, "positions/otc", deal_id
            ))
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            return Err(BrokerageError(format!("edit_position fail with status: {} message: {} deal id: {}", status, message, deal_id)));
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;

        return Ok(());
    }
}
