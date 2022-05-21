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
use std::sync::{Arc};
use tokio::sync::{Mutex};

pub mod models;

pub struct IgRestClient {
    client: Client,
    session: Arc<Mutex<SessionState>>,
    connection_details: ConnectionDetails,
}

impl IgRestClient {
    pub fn new(session: Arc<Mutex<SessionState>>, connection_details: ConnectionDetails) -> Self {
        Self {
            session,
            connection_details,
            client: Client::builder().build().unwrap(),
        }
    }

    pub async fn create_session(&mut self) -> Result<CreateSessionResponseV2, BrokerageError> {
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
            let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
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
        *self.session.lock().await = SessionState::HasSession {
            xst,
            cst,
            lightstreamer_endpoint: res.lightstreamer_endpoint.clone(),
            account: self.connection_details.account.clone(),
        };

        Ok(res)
    }

    pub async fn fetch_data(
        &self,
        start: &str,
        end: &str,
    ) -> Result<FetchDataResponse, BrokerageError> {
        let session = &*self.session.lock().await;
        if let SessionState::HasSession {
            ref xst, ref cst, ..
        } = session
        {
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
                return Err(BrokerageError::CoreBrokerageError);
            }

            let res = res
                .json::<FetchDataResponse>()
                .await
                .map_err(|e| BrokerageError::CoreBrokerageError)?;

            return Ok(res);
        }
        Err(BrokerageError::CoreBrokerageError)
    }

    pub async fn open_working_order(
        &self,
        direction: Direction,
        level: f64,
        deal_reference: &str,
    ) -> Result<(), BrokerageError> {
        let session = &*self.session.lock().await;
        if let SessionState::HasSession {
            ref xst, ref cst, ..
        } = session
        {
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
                .post(format!(
                    "{}{}",
                    self.connection_details.base_url, "workingorders/otc"
                ))
                .headers(headers)
                .json(
                    CreateWorkingOrderRequest::new(direction.into(), level, deal_reference)
                        .borrow(),
                )
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
        return Err(BrokerageError::CoreBrokerageError);
    }

    pub async fn delete_working_order(&self, deal_id: &str) -> Result<(), BrokerageError> {
        let session = &*self.session.lock().await;
        if let SessionState::HasSession {
            ref xst, ref cst, ..
        } = session
        {
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
            headers.insert("Version", "1".parse().unwrap());
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
                    "Failed to close workingorder status {} and with reason: {}",
                    status, message
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
        return Err(BrokerageError::CoreBrokerageError);
    }

    pub async fn edit_position(
        &self,
        deal_id: &str,
        stop_level: f64,
    ) -> Result<(), BrokerageError> {
        let session = &*self.session.lock().await;
        if let SessionState::HasSession {
            ref xst, ref cst, ..
        } = session
        {
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
        Err(BrokerageError::CoreBrokerageError)
    }
}
