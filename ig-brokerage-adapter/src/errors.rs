#[derive(Debug)]
pub enum BrokerageError {
    CoreBrokerageError,
    Error(String),
}

#[derive(Debug)]
pub struct ApiLayerError {
    pub message: String,
    pub status: u16,
}
