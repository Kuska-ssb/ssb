use kuska_sodiumoxide::crypto::{hash::sha256, sign::ed25519};

use super::error::{Error, Result};

pub const CURVE_ED25519_SUFFIX: &str = ".ed25519";
pub const ED25519_SIGNATURE_SUFFIX: &str = ".sig.ed25519";
pub const SHA256_SUFFIX: &str = ".sha256";

pub trait ToSodiumObject {
    fn to_ed25519_pk(&self) -> Result<ed25519::PublicKey>;
    fn to_ed25519_pk_no_suffix(&self) -> Result<ed25519::PublicKey>;
    fn to_ed25519_sk(&self) -> Result<ed25519::SecretKey>;
    fn to_ed25519_sk_no_suffix(&self) -> Result<ed25519::SecretKey>;
    fn to_ed25519_signature(&self) -> Result<ed25519::Signature>;
    fn to_sha256(&self) -> Result<sha256::Digest>;
}

pub trait ToSsbId {
    fn to_ssb_id(&self) -> String;
}

impl<'a> ToSsbId for ed25519::PublicKey {
    fn to_ssb_id(&self) -> String {
        format!("{}{}", base64::encode(self), CURVE_ED25519_SUFFIX)
    }
}

impl<'a> ToSsbId for ed25519::SecretKey {
    fn to_ssb_id(&self) -> String {
        format!("{}{}", base64::encode(self), CURVE_ED25519_SUFFIX)
    }
}

impl<'a> ToSsbId for sha256::Digest {
    fn to_ssb_id(&self) -> String {
        format!("{}{}", base64::encode(self), SHA256_SUFFIX)
    }
}

impl ToSodiumObject for str {
    fn to_ed25519_pk(self: &str) -> Result<ed25519::PublicKey> {
        if !self.ends_with(CURVE_ED25519_SUFFIX) {
            return Err(Error::InvalidSuffix);
        }

        let key_len = self.len() - CURVE_ED25519_SUFFIX.len();
        let bytes = base64::decode(&self[..key_len])?;

        ed25519::PublicKey::from_slice(&bytes).ok_or_else(|| Error::BadPublicKey)
    }
    fn to_ed25519_pk_no_suffix(self: &str) -> Result<ed25519::PublicKey> {
        let bytes = base64::decode(&self)?;

        ed25519::PublicKey::from_slice(&bytes).ok_or_else(|| Error::BadPublicKey)
    }

    fn to_ed25519_sk(self: &str) -> Result<ed25519::SecretKey> {
        if !self.ends_with(CURVE_ED25519_SUFFIX) {
            return Err(Error::InvalidSuffix);
        }

        let key_len = self.len() - CURVE_ED25519_SUFFIX.len();
        let bytes = base64::decode(&self[..key_len])?;

        ed25519::SecretKey::from_slice(&bytes).ok_or_else(|| Error::BadSecretKey)
    }

    fn to_ed25519_sk_no_suffix(self: &str) -> Result<ed25519::SecretKey> {
        let bytes = base64::decode(&self[..])?;

        ed25519::SecretKey::from_slice(&bytes).ok_or_else(|| Error::BadSecretKey)
    }

    fn to_sha256(self: &str) -> Result<sha256::Digest> {
        if !self.ends_with(SHA256_SUFFIX) {
            return Err(Error::InvalidSuffix);
        }
        let key = base64::decode(&self[..self.len() - SHA256_SUFFIX.len()])?;

        sha256::Digest::from_slice(&key).ok_or(Error::InvalidDigest)
    }
    fn to_ed25519_signature(self: &str) -> Result<ed25519::Signature> {
        if !self.ends_with(ED25519_SIGNATURE_SUFFIX) {
            return Err(Error::InvalidSuffix);
        }
        let signature = base64::decode(&self[..self.len() - ED25519_SIGNATURE_SUFFIX.len()])?;

        ed25519::Signature::from_slice(&signature).ok_or(Error::CannotCreateSignature)
    }
}
