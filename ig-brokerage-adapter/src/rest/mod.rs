use crate::errors::ApiLayerError;
use crate::rest::models::{
    AccessTokenResponse, ClosePositionRequest, CreateDeletePositionResponse, CreateSessionRequest,
    CreateSessionRequestV2, CreateSessionResponse, CreateSessionResponseV2,
    CreateWorkingOrderRequest, EditPositionRequest, FetchDataResponse, OpenPositionRequest,
    RefreshTokenRequest,
};
use crate::{BrokerageError, ConnectionDetails, SessionState};
use bfg_core::models::Direction;
use log::{error, info, warn};
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

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
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            //.json::<ApiResponse>().unwrap();
            error!("Failed to create session with reason: {}", message.clone());

            let err = ApiLayerError { status, message };
            return Err(BrokerageError::CoreBrokerageError);
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
            .map_err(|e| BrokerageError::CoreBrokerageError)?;

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
    pub async fn fetch_data(
        &self,
        start: &str,
        end: &str,
    ) -> Result<FetchDataResponse, BrokerageError> {
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
        let resource = format!("prices/IX.D.DAX.IFMM.IP/MINUTE/{start}/{end}");
        let url = format!("{}{}", self.connection_details.base_url, resource);
        let res = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            error!("Failed to get data with reason: {}", message);

            let err = ApiLayerError { status, message };
            println!("Error {:?}", err);
            return Err(BrokerageError::CoreBrokerageError);
        }

        let res = res
            .json::<FetchDataResponse>()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;

        return Ok(res);
    }

    pub async fn open_working_order(
        &self,
        direction: Direction,
        level: f64,
        deal_reference: &str,
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
            CreateWorkingOrderRequest::new(direction.clone().into(), level, deal_reference);
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
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
            error!("Failed to open position with reason: {}", message.clone());

            let err = ApiLayerError { status, message };
            println!(
                "Create working order error {:?} request: {:?}",
                err, request_body
            );
            return Err(BrokerageError::CoreBrokerageError);
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;

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
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap();
            error!(
                "Failed to close workingorder for deal id {} with status {} and with reason: {}",
                deal_id, status, message
            );

            let err = ApiLayerError { status, message };
            return Ok(()); // If i try to close a position that already is closed things crash so i always say ok even if not
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        return Ok(());
    }

    pub async fn edit_position(
        &self,
        deal_id: &str,
        stop_level: f64,
    ) -> Result<(), BrokerageError> {
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
        let res = self
            .client
            .put(format!(
                "{}{}/{}",
                self.connection_details.base_url, "positions/otc", deal_id
            ))
            .headers(headers)
            .json(EditPositionRequest::new(stop_level).borrow())
            .send()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;
        if !res.status().is_success() {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
            error!("Failed to open position with reason: {}", message.clone());

            let err = ApiLayerError { status, message };
            return Err(BrokerageError::CoreBrokerageError);
        }

        let res = res
            .json::<CreateDeletePositionResponse>()
            .await
            .map_err(|e| BrokerageError::CoreBrokerageError)?;

        return Ok(());
    }
}
