mod account;
mod betting;
mod identity;
mod login;

pub use login::*;
pub use identity::*;
pub use betting::*;
pub use account::*;

	// login_url       = "https://identitysso-cert.betfair.se/api/"
	// identity_url    = "https://identitysso.betfair.se/api/"
	// api_betting_url = "https://api.betfair.com/exchange/betting/json-rpc/v1"
	// api_account_url = "https://api.betfair.com/exchange/account/json-rpc/v1"
	// stream_url      = "stream-api.betfair.com:443"

