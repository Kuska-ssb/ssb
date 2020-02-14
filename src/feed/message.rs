use std::str::FromStr;
use std::time::SystemTime;

use serde_json::Value;
use sodiumoxide::crypto::sign::ed25519;

use super::error::{Error, Result};
use super::{ssb_sha256, stringify_json};
use crate::crypto::ToSodiumObject;
use crate::patchwork::IdentitySecret;

const MSG_PREVIOUS: &str = "previous";
const MSG_AUTHOR: &str = "author";
const MSG_SEQUENCE: &str = "sequence";
const MSG_TIMESTAMP: &str = "timestamp";
const MSG_HASH: &str = "hash";
const MSG_CONTENT: &str = "content";
const MSG_SIGNATURE: &str = "signature";

macro_rules! cast {
    ($input:expr,$pth:path) => {
        match $input {
            Some($pth(x)) => Ok(x),
            _ => Err(Error::InvalidJson),
        };
    };
}

macro_rules! cast_opt {
    ($input:expr,$pth:path) => {
        match $input {
            None => Ok(None),
            Some(Value::Null) => Ok(None),
            Some($pth(x)) => Ok(Some(x)),
            _ => Err(Error::InvalidJson),
        };
    };
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub value: serde_json::Value,
}

impl Message {
    pub fn new(prev: Option<&Message>, identity: &IdentitySecret, content: Value) -> Result<Self> {
        let mut value: serde_json::Map<String, Value> = serde_json::Map::new();
        if let Some(prev) = prev {
            value.insert(MSG_PREVIOUS.to_string(), Value::String(prev.id()?));
            value.insert(
                MSG_SEQUENCE.to_string(),
                Value::Number(serde_json::Number::from(prev.sequence() + 1)),
            );
        } else {
            value.insert(
                MSG_SEQUENCE.to_string(),
                Value::Number(serde_json::Number::from(1)),
            );
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis() as u64;

        let timestamp = Value::Number(serde_json::Number::from(timestamp));

        value.insert(MSG_AUTHOR.to_string(), Value::String(identity.id.clone()));
        value.insert(MSG_TIMESTAMP.to_string(), timestamp);
        value.insert(MSG_HASH.to_string(), Value::String("sha256".to_string()));
        value.insert(MSG_CONTENT.to_string(), content);

        let value = Value::Object(value);
        let to_sign_text = stringify_json(&value)?;
        let mut value = cast!(Some(value), Value::Object)?;

        let signature = ed25519::sign_detached(to_sign_text.as_bytes(), &identity.sk);
        value.insert(
            MSG_SIGNATURE.to_string(),
            Value::String(format!("{}.sig.ed25519", base64::encode(&signature))),
        );

        Ok(Message {
            value: Value::Object(value),
        })
    }

    pub fn from_slice(s: &[u8]) -> Result<Self> {
        Self::from_value(serde_json::from_slice(&s)?)
    }

    pub fn from_value(v: Value) -> Result<Self> {
        let mut v = cast!(Some(v), Value::Object)?;

        // check if ok
        cast_opt!(v.get(MSG_PREVIOUS), Value::String)?;
        cast!(v.get(MSG_SEQUENCE), Value::Number)?;
        cast!(v.get(MSG_SEQUENCE), Value::Number)?;
        cast!(v.get(MSG_TIMESTAMP), Value::Number)?;
        cast!(v.get(MSG_HASH), Value::String)?;
        v.get(MSG_CONTENT).ok_or(Error::InvalidJson)?;

        // verify signature
        let signature = cast!(v.remove(MSG_SIGNATURE), Value::String)?;
        let author = cast!(v.get(MSG_AUTHOR), Value::String)?;
        let sig = signature.to_ed25519_signature()?;
        let signer = author[1..].to_ed25519_pk()?;

        let value = Value::Object(v);
        let signed_text = stringify_json(&value)?;
        if !ed25519::verify_detached(&sig, &signed_text.as_ref(), &signer) {
            return Err(Error::InvalidSignature);
        }

        // put signature back
        let mut v = cast!(Some(value), Value::Object)?;
        v.insert(MSG_SIGNATURE.to_string(), Value::String(signature));

        Ok(Message {
            value: Value::Object(v),
        })
    }

    pub fn id(&self) -> Result<String> {
        let digest = base64::encode(&ssb_sha256(&self.value)?);
        Ok(format!("%{}.sha256", digest))
    }

    pub fn previous(&self) -> Option<&String> {
        cast_opt!(self.value.get(MSG_PREVIOUS), Value::String).unwrap()
    }
    pub fn author(&self) -> &String {
        cast!(self.value.get(MSG_AUTHOR), Value::String).unwrap()
    }

    pub fn sequence(&self) -> u64 {
        cast!(self.value.get(MSG_SEQUENCE), Value::Number)
            .unwrap()
            .as_u64()
            .unwrap()
    }

    pub fn timestamp(&self) -> f64 {
        cast!(self.value.get(MSG_TIMESTAMP), Value::Number)
            .unwrap()
            .as_f64()
            .unwrap()
    }

    pub fn hash(&self) -> &String {
        cast!(self.value.get(MSG_HASH), Value::String).unwrap()
    }

    pub fn content(&self) -> &Value {
        self.value.get(MSG_CONTENT).unwrap()
    }

    pub fn signature(&self) -> &String {
        cast!(self.value.get(MSG_SIGNATURE), Value::String).unwrap()
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Message::from_value(serde_json::from_str::<Value>(&s)?)
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        serde_json::to_string(&self.value).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_known_msg_integrity() -> Result<()> {
        let message_id = "%Cg0ZpZ8cV85G8UIIropgBOvM8+Srlv9LSGDNGnpdK44=.sha256";
        let message = r#"{"previous":"%seUEAo7PTyA7vNwnOrmGIsUFfpyRzOvzGVv1QCb/Fz8=.sha256","author":"@BIbVppzlrNiRJogxDYz3glUS7G4s4D4NiXiPEAEzxdE=.ed25519","sequence":37,"timestamp":1439392020612,"hash":"sha256","content":{"type":"post","text":"@paul real time replies didn't work.","repliesTo":"%xWKunF6nXD7XMC+D4cjwDMZWmBnmRu69w9T25iLNa1Q=.sha256","mentions":["%7UKRfZb2u8al4tYWHqM55R9xpE/KKVh9U0M6BdugGt4=.sha256"],"recps":[{"link":"@hxGxqPrplLjRG2vtjQL87abX4QKqeLgCwQpS730nNwE=.ed25519","name":"paul"}]},"signature":"gGxSPdBJZxp6x5f3HzQGoQSeSdh/C5AtymIn+miWa+lcC6DdqpRSgaeH9KHeLf+/CKhU6REYIpWaLr4CKDMfCg==.sig.ed25519"}"#;
        let msg = Message::from_str(&message)?;
        assert_eq!(msg.id()?, message_id);
        Ok(())
    }

    #[test]
    fn test_sign_verify() -> Result<()> {
        let content = Value::String("thistest".to_string());
        let id = IdentitySecret::new();
        let msg1 = Message::new(None, &id, content.clone())?.to_string();
        let msg1 = Message::from_str(&msg1)?;
        let msg2 = Message::new(Some(&msg1), &id, content)?.to_string();
        Message::from_str(&msg2)?;
        Ok(())
    }
}
