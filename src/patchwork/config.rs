use std::io;
use std::string::ToString;

use sodiumoxide::crypto::sign::ed25519;
use sodiumoxide::crypto::auth;

use crate::crypto::ToSodiumObject;

const CURVE_ED25519 : &str = "ed25519";
pub const SSB_NET_ID : &str = "d4a1cb88a66f02f8db635ce26441cc5dac1b08420ceaac230839b755845a9ffb";

#[derive(Debug)]
pub struct IdentitySecret {
    pub id: String,
    pub pk: ed25519::PublicKey,
    pub sk: ed25519::SecretKey,
}

#[derive(Deserialize)]
struct JsonSSBSecret {
    id: String,
    curve: String,
    public: String,
    private: String,
}

pub fn ssb_net_id() -> auth::Key {
    auth::Key::from_slice(&hex::decode(SSB_NET_ID).unwrap()).unwrap()
}

fn to_ioerr<T: ToString>(err: T) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

impl IdentitySecret {

    pub fn new() -> IdentitySecret {
        let (pk, sk) = ed25519::gen_keypair();
        IdentitySecret {
            pk, sk,
            id  : format!("@{}.{}",base64::encode(&pk),CURVE_ED25519),
        }
    }

    pub fn from_local_config() -> io::Result<IdentitySecret> {
        if let Some(home_dir) = dirs::home_dir() {
            let local_key_file = format!("{}/.ssb/secret",home_dir.to_string_lossy());

            std::fs::read_to_string(local_key_file)
                .and_then(IdentitySecret::from_config)

        } else {
            Err(to_ioerr("cannot retrieve home folder"))
        }
    }

    pub fn from_config<T : AsRef<str>>(config: T) -> io::Result<IdentitySecret> {

        // strip all comments
        let json = config.as_ref()
            .lines()
            .filter(|line| !line.starts_with("#"))
            .collect::<Vec<_>>()
            .join("");

        // parse json
        let secret : JsonSSBSecret = serde_json::from_str(json.as_ref())
            .map_err(to_ioerr)?;

        if secret.curve != CURVE_ED25519 {
            return Err(to_ioerr("invalid curve"));
        }
        
        Ok(IdentitySecret {
            id : secret.id,
            pk : secret.public.to_ed25519_pk()?,
            sk : secret.private.to_ed25519_sk()?,
        })
    } 
}