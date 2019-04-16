use reqwest::{Client, Method, Request, Result, Url, StatusCode, Body};
use reqwest::header::{HeaderMap};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use serde_json::json;

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
    token: Option<String>,
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
    pub fn new<T>(api_key: T, account: T) -> Self
    where
        T: ToString,
    {
        let client = Client::new();
        Self {
            account: account.to_string(),
            api_key: api_key.to_string(),
            token: None,
            client: Rc::new(client),
        }
    }

    /// Change the currently set Authorization Token using a type that can turn
    /// into an &str. Must be a valid API Token for requests to work.
    pub fn set_token<T>(&mut self, token: T)
    where
        T: ToString,
    {
        self.token = Some(token.to_string());
    }

    /// Initialize the client calling login endpoint and setting the oauth token on the client
    pub fn initialize(&mut self, user: String, password: String) -> Result<String> {
        // TODO move this as a object method and return a new instance with token set
        let mut body = Map::new();
        body.insert(String::from("identifier") ,json!(user));
        body.insert(String::from("password") ,json!(password));
        let (_headers, status, res) = self.post(body).session().execute::<Value>()?;
        self.set_token(res.unwrap()["oauthToken"]["access_token"].as_str().unwrap());
        Ok(String::from(status.as_str()))
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
    use std::env;
    #[test]
    fn it_works() {
        let usr = env::var("IG_USER").unwrap();
        let pwd = env::var("IG_PWD").unwrap();
        let api_key = env::var("IG_API_KEY").unwrap();
        let account = env::var("IG_ACCOUNT").unwrap();
        // TODO rethink how to do initialize i dont want to have to do IG mut?
        let mut ig = IG::new(&api_key, &account);
        let status = ig.initialize(usr, pwd);
        println!("The initialize status was {:?}", status);

        let (_headers, status, res) = ig.get().session().execute::<Value>().unwrap();
        println!("The get status: {:?} with body {:?}", status, res);
    }
}
