#[derive(Debug)]
pub enum Error {
    HomeNotFound,
    InvalidConfig,
    Serde(serde_json::Error),
    CryptoFormat(crate::crypto::Error),
    SyncIo(std::io::Error),
}
impl From<crate::crypto::Error> for Error {
    fn from(err: crate::crypto::Error) -> Self {
        Error::CryptoFormat(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::SyncIo(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
