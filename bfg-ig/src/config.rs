pub struct IGConfig {
    usr: String,
    pwd: String,
    api_key: String,
    account: String,
}

impl IGConfig {
    pub fn new(usr: String, pwd: String, api_key: String, account: String) -> IGConfig {
        IGConfig {
            usr,
            pwd,
            api_key,
            account,
        }
    }
}
