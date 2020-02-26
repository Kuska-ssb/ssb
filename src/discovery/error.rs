#[derive(Debug)]
pub enum Error {
    ParseInt(std::num::ParseIntError),
    InvalidInviteCode,
    CryptoFormat(crate::crypto::Error),
}

impl From<crate::crypto::Error> for Error {
    fn from(err: crate::crypto::Error) -> Self {
        Error::CryptoFormat(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseInt(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
