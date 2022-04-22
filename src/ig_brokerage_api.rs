use reqwest::Client;
use crate::ports::{BrokerageApi, Or, OrderDetails};

struct Session {
    token: String,
}

enum ConnectionState {
    NoSession,
    HasSession(Session),
}

pub struct IgBrokerageApi {
    pub http_client: reqwest::Client,
    state: ConnectionState,
}

impl BrokerageApi for IgBrokerageApi {
    fn get_or(&self) -> Option<Or> {
        // Build request and call do_request
        todo!()
    }

    fn place_order(&self, order: OrderDetails) {
        // Build request and call do_request
        todo!()
    }
}

impl IgBrokerageApi {
    pub fn new() -> IgBrokerageApi {
        IgBrokerageApi {
            http_client: Client::builder().build().unwrap(),
            state: ConnectionState::NoSession
        }
    }

    fn do_request(&self) {
        // Handle retry and error handling etc here
        match self.state {
            ConnectionState::NoSession => todo!(),
            ConnectionState::HasSession(session) => todo!(),
        }
    }
}

/// This testsuite test agains the testenvironment of IG but its real integration tests
/// Is there a way to exclude there tests from ordinary cargo test?
#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[test]
    fn test_save_and_check() {
        let http_client = Client::builder().build().unwrap();
        let sut = IgBrokerageApi {
            http_client,
            state: ConnectionState::NoSession,
        };
    }
}
