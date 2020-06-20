use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("header size too small")]
    HeaderSizeTooSmall,
    #[error("invalid body type: {0}")]
    InvalidBodyType(u8),
    #[error("i/o")]
    Io(#[from] async_std::io::Error),
    #[error("json decoding")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
