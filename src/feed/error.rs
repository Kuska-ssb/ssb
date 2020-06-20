use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("base64 decoding")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("feed digest mismatch")]
    FeedDigestMismatch,
    #[error("time error")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("invalid json")]
    InvalidJson,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("failed to decipher")]
    FailedToDecipher,
    #[error("cannot create key")]
    CannotCreateKey,
    #[error("cannot read nonce")]
    CannotReadNonce,
    #[error("crypto scalar mult failed")]
    CryptoScalarMultFailed,
    #[error("empty plaintext")]
    EmptyPlaintext,
    #[error("bad recipent")]
    BadRecipientCount,
    #[error("invalid key from group")]
    CryptoKeyFromGrupFailed,
    #[error("invalid key format")]
    CryptoFormat(#[from] crate::crypto::Error),
    #[error("invalid json")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
