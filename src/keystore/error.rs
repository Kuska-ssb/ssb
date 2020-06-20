use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("$HOME not found")]
    HomeNotFound,
    #[error("invalid configuration file")]
    InvalidConfig,
    #[error("json deserialization")]
    Serde(#[from] serde_json::Error),
    #[error("crypto format")]
    CryptoFormat(#[from] crate::crypto::Error),
    #[error("i/o")]
    SyncIo(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
