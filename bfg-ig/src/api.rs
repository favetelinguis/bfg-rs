use bfg_core::Config;
use reqwest;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::Response;
use std::collections::HashMap;

pub fn make_request(config: Config) -> Result<Response, Box<std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("identifier", config.username);
    map.insert("password", config.password);

    let client = reqwest::Client::new(); // TODO client should be reused, set keep alive?
    let res = client
        .post("https://demo-api.ig.com/gateway/deal/session")
        .json(&map)
        .header(CONTENT_TYPE, "application/json; charset=UTF-8")
        .header(ACCEPT, "application/json; charset=UTF-8")
        .header("X-IG-API-KEY", config.api_key)
        .header("IG-ACCOUNT-ID", config.account)
        .send()?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_login() {}
}
