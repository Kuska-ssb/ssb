use sodiumoxide::crypto::sign::ed25519;
use sodiumoxide::crypto::hash::sha256;
use async_std::io;
use base64;

use crate::util::to_ioerr;

const CURVE_ED25519_SUFFIX : &str = ".ed25519";
const ED25519_SIGNATURE_SUFFIX : &str = ".sig.ed25519";
const SHA256_SUFFIX : &str = ".sha256";

pub trait ToSodiumObject {
    fn to_ed25519_pk(&self) -> io::Result<ed25519::PublicKey>;
    fn to_ed25519_sk(&self) -> io::Result<ed25519::SecretKey>;
    fn to_ed25519_sk_no_suffix(&self) -> io::Result<ed25519::SecretKey>;
    fn to_ed25519_signature(&self) -> io::Result<ed25519::Signature>;    
    fn to_sha256(&self) -> io::Result<sha256::Digest>;
}

impl ToSodiumObject for str {
    fn to_ed25519_pk(self : &str) -> io::Result<ed25519::PublicKey> {
        if !self.ends_with(CURVE_ED25519_SUFFIX) {
            return Err(to_ioerr("invalid suffix"));
        }
    
        let key_len = self.len()-CURVE_ED25519_SUFFIX.len();
        let bytes = base64::decode(&self[..key_len])
            .map_err(to_ioerr)?;
        
        ed25519::PublicKey::from_slice(&bytes)
            .ok_or_else(|| to_ioerr("bad public key"))
    }
    
    fn to_ed25519_sk(self : &str) -> io::Result<ed25519::SecretKey> {
        if !self.ends_with(CURVE_ED25519_SUFFIX) {
            return Err(to_ioerr("invalid suffix"));
        }
    
        let key_len = self.len()-CURVE_ED25519_SUFFIX.len();
        let bytes = base64::decode(&self[..key_len]).map_err(to_ioerr)?;
    
        ed25519::SecretKey::from_slice(&bytes)
            .ok_or_else(|| to_ioerr("bad secret key"))
    }

    fn to_ed25519_sk_no_suffix(self : &str) -> io::Result<ed25519::SecretKey> {
        let bytes = base64::decode(&self[..]).map_err(to_ioerr)?;
    
        ed25519::SecretKey::from_slice(&bytes)
            .ok_or_else(|| to_ioerr("bad secret key"))
    }

    fn to_sha256(self : &str) -> io::Result<sha256::Digest> {
        if !self.ends_with(SHA256_SUFFIX) {
            return Err(to_ioerr("invalid hash suffix"));
        }
        let key = base64::decode(&self[..self.len()-SHA256_SUFFIX.len()])
            .map_err(to_ioerr)?;

        sha256::Digest::from_slice(&key)
            .ok_or(to_ioerr("cannot create digest"))
    }
    fn to_ed25519_signature(self : &str) -> io::Result<ed25519::Signature> {    
        if !self.ends_with(ED25519_SIGNATURE_SUFFIX) {
            return Err(to_ioerr("invalid signature suffix"));
        }
        let signature = base64::decode(&self[..self.len()-ED25519_SIGNATURE_SUFFIX.len()])
            .map_err(to_ioerr)?;

        ed25519::Signature::from_slice(&signature)
        .ok_or(to_ioerr("cannot create signature"))
    }
}
