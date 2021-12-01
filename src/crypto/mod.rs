mod error;
mod sodium;

pub use error::{Error, Result};
pub use kuska_sodiumoxide::crypto::{hash::sha256, sign::ed25519};
pub use sodium::{
    ToSodiumObject, ToSsbId, CURVE_ED25519_SUFFIX, ED25519_SIGNATURE_SUFFIX, SHA256_SUFFIX,
};
