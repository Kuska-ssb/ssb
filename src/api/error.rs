use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("rpc")]
    Rpc(#[from] crate::rpc::Error),
    #[error("json decode")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
