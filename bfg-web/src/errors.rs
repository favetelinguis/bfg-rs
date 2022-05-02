use std::fmt;
use std::fmt::{Display, Formatter};
use actix_web::{error, HttpResponse, ResponseError};
use actix_web::error::PrivateHelper;
use actix_web::http::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum BfgWebError {
    CustomError1(String),
    NotFound(String),
}

#[derive(Debug, Serialize)]
pub struct BfgWebErrorResponse {
    error_message: String,
}

impl fmt::Display for BfgWebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl BfgWebError {
    fn error_response(&self) -> String {
        match self {
            BfgWebError::CustomError1(msg) => {
                println!("Custom error1 happened: {:?}", msg);
                "Custom error 1".into()
            }
            BfgWebError::NotFound(msg) => {
                println!("Not found error occurred: {:?}", msg);
                msg.into()
            }
        }
    }
}

impl ResponseError for BfgWebError {
    fn status_code(&self) -> StatusCode {
        match self {
            BfgWebError::CustomError1(_) => {
                StatusCode::ALREADY_REPORTED
            }
            BfgWebError::NotFound(_) => StatusCode::NOT_FOUND
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(BfgWebErrorResponse {
            error_message: self.error_response(),
        })
    }
}
// Let each result return BfgWebError and imp a from for the other error like this
// impl From<actix_web::error::Error> for BfgWebError {
//     fn from(err: actix_web::error::Error) -> Self {
//         BfgWebError::NotFound(err.to_string())
//     }
// }
