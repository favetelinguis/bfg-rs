use crate::errors::ApiLayerError;
use crate::rest::models::{
    AccessTokenResponse, ClosePositionRequest, CreateDeletePositionResponse, CreateSessionRequest,
    CreateSessionResponse, OpenPositionRequest, RefreshTokenRequest,
};
use crate::{BrokerageError, ConnectionDetails};
use bfg_core::models::Direction;
use log::{error, info};
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::borrow::Borrow;
use futures_util::future::err;

pub mod models;

pub async fn close_position(
    client: Client,
    connection_details: &ConnectionDetails,
    access_token: &str,
    direction: Direction,
    level: f64,
    deal_reference: String,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("IG-ACCOUNT-ID", connection_details.account.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    headers.insert("_method", "DELETE".parse().unwrap());
    let body =
        serde_json::to_string(ClosePositionRequest::new(direction.into(), 1).borrow()).unwrap();
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "positions/otc"
        ))
        .bearer_auth(access_token)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
        // TODO crash with errorCode: unable to aggregate close positions - no compatible position found
        error!("Failed to close position status {} and with reason: {}", status, message);

        let err = ApiLayerError { status, message };
        return Err(BrokerageError::CoreBrokerageError);
    }

    let res = res
        .json::<CreateDeletePositionResponse>()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;
    Ok(())
}

pub async fn open_position(
    client: Client,
    connection_details: &ConnectionDetails,
    access_token: &str,
    direction: Direction,
    level: f64,
    deal_reference: String,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("IG-ACCOUNT-ID", connection_details.account.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "positions/otc"
        ))
        .bearer_auth(access_token)
        .headers(headers)
        .json::<OpenPositionRequest>(
            OpenPositionRequest::new(direction.into(), level, deal_reference).borrow(),
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

    Ok(())
}

pub async fn create_session(
    client: Client,
    connection_details: &ConnectionDetails,
) -> Result<CreateSessionResponse, BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("IG-ACCOUNT-ID", connection_details.account.parse().unwrap());
    headers.insert("Version", "3".parse().unwrap());
    let res = client
        .post(format!("{}{}", connection_details.base_url, "session"))
        .headers(headers)
        .json::<CreateSessionRequest>(&CreateSessionRequest {
            identifier: connection_details.username.clone(),
            password: connection_details.password.clone(),
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

    let res = res
        .json::<CreateSessionResponse>()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;

    Ok(res)
}

pub async fn get_session(
    client: Client,
    connection_details: &ConnectionDetails,
    access_token: &str,
) -> Result<(String, String), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    let res = client
        .get(format!(
            "{}{}?fetchSessionTokens=true",
            connection_details.base_url, "session"
        ))
        .bearer_auth(access_token)
        .headers(headers)
        .send()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
        error!("Failed to fetch session with reason: {}", message.clone());

        let err = ApiLayerError { status, message };
        return Err(BrokerageError::CoreBrokerageError);
    }

    let xst = res
        .headers()
        .get("x-security-token")
        .unwrap()
        .to_str()
        .unwrap()
        .into();
    let cst = res.headers().get("cst").unwrap().to_str().unwrap().into();
    Ok((xst, cst))
}

pub async fn refresh_token(
    client: Client,
    connection_details: &ConnectionDetails,
    refresh_token: &str,
) -> Result<AccessTokenResponse, BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("IG-ACCOUNT-ID", connection_details.account.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "session/refresh-token"
        ))
        .headers(headers)
        .json::<RefreshTokenRequest>(
            RefreshTokenRequest {
                refresh_token: refresh_token.to_string(),
            }
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
        .json::<AccessTokenResponse>()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;

    Ok(res)
}
