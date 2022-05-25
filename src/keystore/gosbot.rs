//! Read and write a go-sbot secret (identity) file.

#![warn(missing_docs)]

use async_std::{
    io::{Read, Write},
    prelude::*,
};

use super::{
    error::{Error, Result},
    util, JsonSSBSecret, OwnedIdentity, CURVE_ED25519,
};
use crate::crypto::{ToSodiumObject, ToSsbId};

/// Return an `OwnedIdentity` from the local go-sbot secret file with a custom path
pub async fn from_custom_gosbot_keypath(local_key_file: String) -> Result<OwnedIdentity> {
    let mut file = async_std::fs::File::open(local_key_file).await?;
    read_gosbot_config(&mut file).await
}

/// Return an `OwnedIdentity` from the local go-sbot secret file in the default location.
pub async fn from_gosbot_local() -> Result<OwnedIdentity> {
    let home_dir = dirs::home_dir().ok_or(Error::HomeNotFound)?;
    let local_key_file = format!("{}/.ssb-go/secret", home_dir.to_string_lossy());
    from_custom_gosbot_keypath(local_key_file).await
}

/// Read the contents of the go-sbot secret file, deserialize into a
/// `JsonSSBSecret` and return an `OwnedIdentity`.
pub async fn read_gosbot_config<R: Read + Unpin>(reader: &mut R) -> Result<OwnedIdentity> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf).await?;

    // parse json
    let secret: JsonSSBSecret = serde_json::from_str(buf.as_ref()).map_err(util::to_io_error)?;

    if secret.curve != CURVE_ED25519 {
        return Err(Error::InvalidConfig);
    }

    Ok(OwnedIdentity {
        id: secret.id,
        pk: secret.public.to_ed25519_pk()?,
        sk: secret.private.to_ed25519_sk()?,
    })
}

/// Write an `OwnedIdentity`.
pub async fn write_gosbot_config<W: Write + Unpin>(
    id: &OwnedIdentity,
    writer: &mut W,
) -> Result<()> {
    let json = JsonSSBSecret {
        curve: CURVE_ED25519.to_owned(),
        id: id.id.clone(),
        private: id.sk.to_ssb_id(),
        public: id.pk.to_ssb_id(),
    };
    let encoded = serde_json::to_vec(&json)?;
    Ok(writer.write_all(&encoded).await?)
}

#[cfg(test)]
mod test {
    use super::*;
    use async_std::io::Cursor;

    // ssb secret file contents, as formatted by go-sbot
    const SECRET: &str = r#"{"curve":"ed25519","id":"@1vxS6DMi7z9uJIQG33W7mlsv21GZIbOpmWE1QEcn9oY=.ed25519","private":"F9bw6dPLaHR89hg6Q2dRmoNHHjm+COI53L0kdV3Y4w3W/FLoMyLvP24khAbfdbuaWy/bUZkhs6mZYTVARyf2hg==.ed25519","public":"1vxS6DMi7z9uJIQG33W7mlsv21GZIbOpmWE1QEcn9oY=.ed25519"}"#;

    #[async_std::test]
    async fn test_gosbot_secret() -> Result<()> {
        let mut secret_bytes = SECRET.as_bytes();
        let read_secret_output = read_gosbot_config(&mut secret_bytes).await?;
        let expected = OwnedIdentity {
            id: "@1vxS6DMi7z9uJIQG33W7mlsv21GZIbOpmWE1QEcn9oY=.ed25519".to_owned(),
            sk: "F9bw6dPLaHR89hg6Q2dRmoNHHjm+COI53L0kdV3Y4w3W/FLoMyLvP24khAbfdbuaWy/bUZkhs6mZYTVARyf2hg==.ed25519".to_ed25519_sk()?,
            pk: "1vxS6DMi7z9uJIQG33W7mlsv21GZIbOpmWE1QEcn9oY=.ed25519".to_ed25519_pk()?
        };
        assert_eq!(expected, read_secret_output);

        // create a Cursor which wraps an in-memory buffer (implements `Write`)
        let mut secret_buffer = Cursor::new(Vec::new());
        // write the `OwnedIdentity` from `read_gosbot_config()` to the buffer
        write_gosbot_config(&read_secret_output, &mut secret_buffer).await?;
        // retrieve the value from inside the Cursor
        let secret_vector = secret_buffer.into_inner();
        // convert the byte slice to a string slice
        let write_secret_output = std::str::from_utf8(&secret_vector).unwrap();
        assert_eq!(SECRET, write_secret_output);

        Ok(())
    }
}
