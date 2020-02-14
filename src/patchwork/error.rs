#[derive(Debug)]
pub enum Error {
    InvalidSequenceNo,
    ServerMessage(String),
    Rpc(crate::rpc::Error),
    InvalidInviteCode,
    HomeNotFound,
    InvalidConfig,
    Json(serde_json::Error),
    ParseInt(std::num::ParseIntError),
    CryptoFormat(crate::crypto::Error),
    Io(std::io::Error),
    Feed(crate::feed::Error),
}

impl From<crate::rpc::Error> for Error {
    fn from(err: crate::rpc::Error) -> Self {
        Error::Rpc(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseInt(err)
    }
}

impl From<crate::crypto::Error> for Error {
    fn from(err: crate::crypto::Error) -> Self {
        Error::CryptoFormat(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<crate::feed::Error> for Error {
    fn from(err: crate::feed::Error) -> Self {
        Error::Feed(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
