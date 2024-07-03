// TODO env should prob not be pub
pub mod env;

pub mod rest;

use anyhow::{Context, Result};

pub struct Session {
    // try read file from disk
    // if exist read json and try some ping endpoint on account
    // if ping fail do login and create new file then try to ping again
    // if that ping fails panic
    token: Option<String>,
}

// TODO start keep alive on this and impl the drop trait on this to stop keep alive once
// i make the long running version.
// TODO is it there we should keep track of rate limit toward the api?
impl Session {
    // pub fn new() -> Result<> {
    // env::ConnectionConfig::new()
    //     rest::login(, , , )
    //     .
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let mut result = Vec::new();
        // find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut result).unwrap();
        // assert_eq!(result, b"lorem ipsum\n");
    }
}
