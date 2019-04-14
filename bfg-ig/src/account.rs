use bfg_core::account::AccountProvider;

use super::config::IGConfig;

pub struct IGAccountProvider {
    config: IGConfig,
}

impl IGAccountProvider {
    pub fn new(config: IGConfig) -> IGAccountProvider {
        IGAccountProvider { config }
    }
}

impl AccountProvider for IGAccountProvider {
    fn get_account(&self) -> String {
        String::from("Hello from IG!")
    }
}
