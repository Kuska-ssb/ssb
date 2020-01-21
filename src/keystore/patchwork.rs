use std::io;
use std::string::ToString;

use crate::crypto::ToSodiumObject;

use super::error::{Error, Result};
use super::OwnedIdentity;

pub const CURVE_ED25519: &str = "ed25519";

#[derive(Deserialize)]
struct JsonSSBSecret {
    id: String,
    curve: String,
    public: String,
    private: String,
}

fn to_ioerr<T: ToString>(err: T) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

pub fn from_patchwork_local() -> Result<OwnedIdentity> {
    let home_dir = dirs::home_dir().ok_or(Error::HomeNotFound)?;
    let local_key_file = format!("{}/.ssb/secret", home_dir.to_string_lossy());
    let content = std::fs::read_to_string(local_key_file)?;
    Ok(from_patchwork_config(content)?)
}

pub fn from_patchwork_config<T: AsRef<str>>(config: T) -> Result<OwnedIdentity> {
    // strip all comments
    let json = config
        .as_ref()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("");

    // parse json
    let secret: JsonSSBSecret = serde_json::from_str(json.as_ref()).map_err(to_ioerr)?;

    if secret.curve != CURVE_ED25519 {
        return Err(Error::InvalidConfig);
    }

    Ok(OwnedIdentity {
        id: secret.id,
        pk: secret.public.to_ed25519_pk()?,
        sk: secret.private.to_ed25519_sk()?,
    })
}
