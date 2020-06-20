use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid integer")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("invalid invite code")]
    InvalidInviteCode,
    #[error("invalid broadcast message")]
    InvalidBroadcastMessage,
    #[error("invalid crypto format")]
    CryptoFormat(#[from] crate::crypto::Error),
    #[error("i/o")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
