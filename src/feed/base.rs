use serde_json::Value;

use super::{
    error::{Error, Result},
    message::Message,
    ssb_sha256,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub key: String,
    pub value: Value,
    pub timestamp: f64,
    pub rts: Option<f64>,
}

impl ToString for Feed {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Feed {
    pub fn into_message(self) -> Result<Message> {
        Message::from_value(self.value)
    }
    pub fn new(m: Message) -> Self {
        let key = format!("%{}.sha256", base64::encode(&ssb_sha256(&m.value).unwrap()));
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Feed {
            key,
            value: m.value,
            timestamp,
            rts: None,
        }
    }
    pub fn from_slice(s: &[u8]) -> Result<Self> {
        let feed: Feed = serde_json::from_slice(s)?;
        let digest = format!("%{}.sha256", base64::encode(&ssb_sha256(&feed.value)?));

        if digest != feed.key {
            return Err(Error::FeedDigestMismatch);
        }

        Ok(feed)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_verify_feed_integrity() -> Result<()> {
        let feed = r#"{"key":"%Cg0ZpZ8cV85G8UIIropgBOvM8+Srlv9LSGDNGnpdK44=.sha256","value":{"previous":"%seUEAo7PTyA7vNwnOrmGIsUFfpyRzOvzGVv1QCb/Fz8=.sha256","author":"@BIbVppzlrNiRJogxDYz3glUS7G4s4D4NiXiPEAEzxdE=.ed25519","sequence":37,"timestamp":1439392020612,"hash":"sha256","content":{"type":"post","text":"@paul real time replies didn't work.","repliesTo":"%xWKunF6nXD7XMC+D4cjwDMZWmBnmRu69w9T25iLNa1Q=.sha256","mentions":["%7UKRfZb2u8al4tYWHqM55R9xpE/KKVh9U0M6BdugGt4=.sha256"],"recps":[{"link":"@hxGxqPrplLjRG2vtjQL87abX4QKqeLgCwQpS730nNwE=.ed25519","name":"paul"}]},"signature":"gGxSPdBJZxp6x5f3HzQGoQSeSdh/C5AtymIn+miWa+lcC6DdqpRSgaeH9KHeLf+/CKhU6REYIpWaLr4CKDMfCg==.sig.ed25519"},"timestamp":1573574678194,"rts":1439392020612}"#;
        Feed::from_slice(feed.as_bytes())?;
        Ok(())
    }
}
