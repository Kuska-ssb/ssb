#[derive(Debug)]
pub enum Error {
    Base64Decode(base64::DecodeError),
    FeedDigestMismatch,
    SystemTimeError(std::time::SystemTimeError),
    InvalidJson,
    InvalidSignature,
    FailedToDecipher,
    CannotCreateKey,
    CannotReadNonce,
    CryptoScalarMultFailed,
    EmptyPlaintext,
    BadRecipientCount,
    CryptoKeyFromGrupFailed,
    CryptoFormat(crate::crypto::Error),
    Json(serde_json::Error),
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::Base64Decode(err)
    }
}
impl From<crate::crypto::Error> for Error {
    fn from(err: crate::crypto::Error) -> Self {
        Error::CryptoFormat(err)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::SystemTimeError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error { }

pub type Result<T> = std::result::Result<T, Error>;

