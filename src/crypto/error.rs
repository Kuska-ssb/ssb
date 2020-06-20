use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error decoding base64")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("bad public key")]
    BadPublicKey,
    #[error("bad secret key")]
    BadSecretKey,
    #[error("invalid digest")]
    InvalidDigest,
    #[error("invalid suffix")]
    InvalidSuffix,
    #[error("cannot create signature")]
    CannotCreateSignature,
}

pub type Result<T> = std::result::Result<T, Error>;
