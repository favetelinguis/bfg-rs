pub trait AccountProvider {
    fn get_account(&self) -> String;
}
