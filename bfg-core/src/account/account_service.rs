use super::account_provider::AccountProvider;

pub struct AccountService {
    provider: Box<dyn AccountProvider>,
}

impl AccountService {
    pub fn new(provider: Box<AccountProvider>) -> AccountService {
        AccountService { provider }
    }

    pub fn get_account(&self) -> String {
        self.provider.get_account()
    }
}
