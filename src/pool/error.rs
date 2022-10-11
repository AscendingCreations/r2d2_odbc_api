use thiserror::Error;

#[derive(Error, Debug)]
pub enum OdbcError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    OdbcApi(#[from] odbc_api::Error),
    #[error("Lock is poisoned {msg} ")]
    LockError { msg: String },
    #[error("unhandled error")]
    Unknown,
}
