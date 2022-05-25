pub mod order;
pub mod system;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderReference {
    OVER_LONG,
    BETWEEN_LONG,
    BETWEEN_SHORT,
    UNDER_SHORT,
}

pub enum Event {
    ConfirmationOpenAccepted, //{deal_id: String, deal_reference: WorkingOrderReference, level: f64},
    ConfirmationOpenRejected,// {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationCloseAccepted,// {deal_reference: WorkingOrderReference},
    ConfirmationCloseRejected,// {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationAmendedAccepted,// {deal_reference: WorkingOrderReference, level: f64},
    ConfirmationAmendedRejected, //{deal_reference: WorkingOrderReference, reason: String},
    PositionUpdateOpen, // {deal_reference: WorkingOrderReference, level: f64},
    PositionUpdateDelete, // {deal_reference: WorkingOrderReference, level: f64},
    Market(),
    Account(),
}

pub enum Command {
    FetchData,//(FetchDataDetails),
    CreateWorkingOrder,//(WorkingOrderDetails),
    CancelWorkingOrder,//(String),          // deal_id
    UpdatePosition,//(String, f64), // deal_id, stop_level
    PublishTradeResults,
}