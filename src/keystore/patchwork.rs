use async_std::{
    io::{Read, Write},
    prelude::*,
};

use std::string::ToString;

use super::{
    error::{Error, Result},
    OwnedIdentity,
};
use crate::crypto::{ToSodiumObject, ToSsbId};
use serde_json::to_vec_pretty;

pub const CURVE_ED25519: &str = "ed25519";

#[derive(Serialize, Deserialize)]
struct JsonSSBSecret {
    id: String,
    curve: String,
    public: String,
    private: String,
}

fn to_io_error<T: ToString>(err: T) -> async_std::io::Error {
    async_std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}

pub async fn from_patchwork_local() -> Result<OwnedIdentity> {
    let home_dir = dirs::home_dir().ok_or(Error::HomeNotFound)?;
    let local_key_file = format!("{}/.ssb/secret", home_dir.to_string_lossy());
    let mut file = async_std::fs::File::open(local_key_file).await?;
    read_patchwork_config(&mut file).await
}

pub async fn read_patchwork_config<R: Read + Unpin>(reader: &mut R) -> Result<OwnedIdentity> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf).await?;

    let json = buf
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("");

    // parse json
    let secret: JsonSSBSecret = serde_json::from_str(json.as_ref()).map_err(to_io_error)?;

    if secret.curve != CURVE_ED25519 {
        return Err(Error::InvalidConfig);
    }

    Ok(OwnedIdentity {
        id: secret.id,
        pk: secret.public.to_ed25519_pk()?,
        sk: secret.private.to_ed25519_sk()?,
    })
}

pub async fn write_patchwork_config<W: Write + Unpin>(
    id: &OwnedIdentity,
    writer: &mut W,
) -> Result<()> {
    let json = JsonSSBSecret {
        id: id.id.clone(),
        curve: CURVE_ED25519.to_owned(),
        public: id.pk.to_ssb_id(),
        private: id.sk.to_ssb_id(),
    };
    let encoded = to_vec_pretty(&json)?;
    Ok(writer.write_all(&encoded).await?)
}
