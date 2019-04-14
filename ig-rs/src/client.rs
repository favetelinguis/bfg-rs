use reqwest::{Client, Method, Request, Result, Url, StatusCode, Body};
use reqwest::header::{HeaderMap};
use serde::de::DeserializeOwned;

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use serde_json;
use serde::Serialize;

use crate::login;

/// All GET based queries can be constructed from this type
new_type!(GetQueryBuilder);

/// All PUT based queries can be constructed from this type
new_type!(PutQueryBuilder);

/// All POST based queries can be constructed from this type
new_type!(PostQueryBuilder);

/// All DELETE based queries can be constructed from this type
new_type!(DeleteQueryBuilder);

/// All PATCH based queries can be constructed from this type
new_type!(PatchQueryBuilder);

pub struct IG {
    // TODO I could have an enum here saying if we are logged, mayby just Optional key
    account: String,
    api_key: String,
    token: String,
    client: Rc<Client>,
}

pub trait Executor {
    fn execute<T>(self) -> Result<(HeaderMap, StatusCode, Option<T>)>
        where
            T: DeserializeOwned;
}

impl IG {
    /// Create a new IG client struct. It takes a type that can convert into
    /// an &str (`String` or `Vec<u8>` for example). As long as the function is
    /// given a valid API Token your requests will work.
    pub fn new<T>(token: T, api_key: T, account: T) -> Self
    where
        T: ToString,
    {
        let client = Client::new();
        Self {
            account: account.to_string(),
            api_key: api_key.to_string(),
            token: token.to_string(),
            client: Rc::new(client),
        }
    }

    /// Get the currently set Authorization Token
    pub fn get_token(&self) -> &str {
        &self.token
    }

    /// Change the currently set Authorization Token using a type that can turn
    /// into an &str. Must be a valid API Token for requests to work.
    pub fn set_token<T>(&mut self, token: T)
    where
        T: ToString,
    {
        self.token = token.to_string();
    }

    pub fn get(&self) -> GetQueryBuilder {
        self.into()
    }

    pub fn post<T>(&self, body: T) -> PostQueryBuilder
    where
        T: Serialize,
    {
        let mut qb: PostQueryBuilder = self.into();
        let json: Body = serde_json::to_vec(&body).unwrap().into();
        qb.request.borrow_mut().body_mut().replace(json);
        qb
    }
}

// From derivations of IG to the given type using a certain
// request method
from!(
    @GetQueryBuilder
        => Method::GET
    @PutQueryBuilder
        => Method::PUT
    @PostQueryBuilder
        => Method::POST
    @PatchQueryBuilder
        => Method::PATCH
    @DeleteQueryBuilder
        => Method::DELETE
);

impl<'g> GetQueryBuilder<'g> {
    func_client!(session, login::get::Login<'g>);
}

impl<'g> PostQueryBuilder<'g> {
    func_client!(session, login::post::Session<'g>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};
    use serde_json::json;
    use std::env;
    #[test]
    fn it_works() {
        let usr = env::var("IG_USER").unwrap();
        let pwd = env::var("IG_PWD").unwrap();
        let api_key = env::var("IG_API_KEY").unwrap();
        let account = env::var("IG_ACCOUNT").unwrap();
        let ig = IG::new("Secret", &api_key, &account);
        let mut body = Map::new();
        body.insert(String::from("identifier") ,json!(usr));
        body.insert(String::from("password") ,json!(pwd));
        // TODO put a method directrly on IG that called initialize that sets post and sets the token
        let res = ig.post(body).session().execute::<Value>();
        println!("{:?}", res);
    }
}
