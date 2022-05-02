use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiResponse(pub String);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: String,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionResponse {
    #[serde(rename = "oauthToken")]
    pub oauth_token: AccessTokenResponse,
    #[serde(rename = "lightstreamerEndpoint")]
    pub lightstreamer_endpoint: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSessionRequest {
    pub identifier: String,
    pub password: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetMarketResponse {
    pub instrument: InstrumentDetails,
    #[serde(rename = "dealingRules")]
    pub dealing_rules: DealingRules
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Unit {
    PERCENTAGE, POINTS
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DealingRule {
    pub unit: Unit,
    pub value: f32

}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DealingRules {
    #[serde(rename = "minNormalStopOrLimitDistance")]
    pub min_normal_stop_or_limit_distance: DealingRule,
    #[serde(rename = "minDealSize")]
    pub min_deal_size: DealingRule,
    #[serde(rename = "minStepDistance")]
    pub min_step_distance: DealingRule,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InstrumentDetails {
    pub name: String,
    # [serde(rename = "valueOfOnePip")]
    pub value_of_one_pip: String,
    # [serde(rename = "onePipMeans")]
    pub one_pip_means: String,
    # [serde(rename = "contractSize")]
    pub contractSize: String,
}
