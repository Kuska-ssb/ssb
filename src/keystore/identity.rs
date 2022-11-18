use crate::crypto::CURVE_ED25519_SUFFIX;
use kuska_sodiumoxide::crypto::sign::ed25519;

/// Ed25519 signature scheme identifier.
pub const CURVE_ED25519: &str = "ed25519";

#[derive(Serialize, Deserialize)]
pub struct JsonSSBSecret {
    pub curve: String,
    pub id: String,
    pub private: String,
    pub public: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OwnedIdentity {
    pub id: String,
    pub pk: ed25519::PublicKey,
    pub sk: ed25519::SecretKey,
}

impl OwnedIdentity {
    pub fn create() -> OwnedIdentity {
        let (pk, sk) = ed25519::gen_keypair();
        OwnedIdentity {
            pk,
            sk,
            id: format!("@{}{}", base64::encode(&pk), CURVE_ED25519_SUFFIX),
        }
    }
}
