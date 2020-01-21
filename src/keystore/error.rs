#[derive(Debug)]
pub enum Error {
    HomeNotFound,
    InvalidConfig,
    CryptoFormat(crate::crypto::Error),
    Io(std::io::Error),
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
