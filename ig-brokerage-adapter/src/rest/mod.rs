use crate::errors::ApiLayerError;
use crate::rest::models::{AccessTokenResponse, ClosePositionRequest, CreateDeletePositionResponse, CreateSessionRequest, CreateSessionRequestV2, CreateSessionResponse, CreateSessionResponseV2, CreateWorkingOrderRequest, EditPositionRequest, FetchDataResponse, OpenPositionRequest, RefreshTokenRequest};
use crate::{BrokerageError, ConnectionDetails};
use bfg_core::models::Direction;
use futures_util::future::err;
use futures_util::TryFutureExt;
use log::{error, info, warn};
use reqwest::header::{HeaderMap, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::borrow::Borrow;

pub mod models;

pub async fn close_position(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
    direction: Direction,
    deal_reference: String,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    headers.insert("_method", "DELETE".parse().unwrap());
    let body =
        serde_json::to_string(ClosePositionRequest::new(direction.into(), 1).borrow()).unwrap();
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "positions/otc"
        ))
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let message = res.text().await.unwrap(); //.json::<ApiResponse>().unwrap();
                                                 // TODO crash with errorCode: unable to aggregate close positions - no compatible position found
        error!(
            "Failed to close position status {} and with reason: {}",
            status, message
        );

        let err = ApiLayerError { status, message };
        // return Err(BrokerageError::CoreBrokerageError);
        return Ok(()); // If i try to close a position that already is closed things crash so i always say ok even if not
    }

    let res = res
        .json::<CreateDeletePositionResponse>()
        .await
        .map_err(|e| BrokerageError::CoreBrokerageError)?;
    Ok(())
}

pub async fn fetch_data(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
    start: &str,
    end: &str,
) -> Result<FetchDataResponse, BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let resource = format!("prices/IX.D.DAX.IFMM.IP/MINUTE/{start}/{end}");
    let url = format!("{}{}", connection_details.base_url, resource);
    let res = client
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

    Ok(res)
}

pub async fn open_position(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
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
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "positions/otc"
        ))
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
) -> Result<(String, String, CreateSessionResponseV2), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let res = client
        .post(format!("{}{}", connection_details.base_url, "session"))
        .headers(headers)
        .json(&CreateSessionRequestV2 {
            encrypted_password: false,
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

    Ok((xst, cst, res))
}

pub async fn get_session(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
) -> Result<(String, String), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    let res = client
        .get(format!(
            "{}{}?fetchSessionTokens=true",
            connection_details.base_url, "session"
        ))
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

pub async fn open_working_order(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
    direction: Direction,
    level: f64,
    deal_reference: &str,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let res = client
        .post(format!(
            "{}{}",
            connection_details.base_url, "workingorders/otc"
        ))
        .headers(headers)
        .json(CreateWorkingOrderRequest::new(direction.into(), level, deal_reference).borrow())
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

pub async fn delete_working_order(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
    deal_id: &str,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "1".parse().unwrap());
    headers.insert("_method", "DELETE".parse().unwrap());
    let res = client
        .post(format!(
            "{}{}/{}",
            connection_details.base_url, "workingorders/otc", deal_id
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
    Ok(())
}

pub async fn edit_position(
    client: Client,
    connection_details: &ConnectionDetails,
    xst: &str,
    cst: &str,
    deal_id: &str,
    stop_level: f64,
) -> Result<(), BrokerageError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json; charset=UTF-8".parse().unwrap());
    headers.insert(
        CONTENT_TYPE,
        "application/json; charset=UTF-8".parse().unwrap(),
    );
    headers.insert("X-IG-API-KEY", connection_details.api_key.parse().unwrap());
    headers.insert("X-SECURITY-TOKEN", xst.parse().unwrap());
    headers.insert("CST", cst.parse().unwrap());
    headers.insert("Version", "2".parse().unwrap());
    let res = client
        .put(format!(
            "{}{}/{}",
            connection_details.base_url, "positions/otc", deal_id
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

    Ok(())
}
