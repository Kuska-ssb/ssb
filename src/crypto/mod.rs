mod error;
mod sodium;

pub use error::{Error, Result};
pub use sodium::{
    ToSodiumObject, ToSsbId, CURVE_ED25519_SUFFIX, ED25519_SIGNATURE_SUFFIX, SHA256_SUFFIX,
};
pub use sodiumoxide::crypto::{hash::sha256, sign::ed25519};
