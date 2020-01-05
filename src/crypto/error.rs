#[derive(Debug)]
pub enum Error {
    Base64Decode(base64::DecodeError),
    BadPublicKey,
    BadSecretKey,
    InvalidDigest,
    InvalidSuffix,
    CannotCreateSignature,
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::Base64Decode(err)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error { }

pub type Result<T> = std::result::Result<T, Error>;

