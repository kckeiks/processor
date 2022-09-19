use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("invalid data")]
    InvalidData,
    #[error("insufficient funds for operation")]
    InsufficientFunds,
    #[error("overflow")]
    Overflow,
    #[error("tx already exists")]
    TxExists,
}
