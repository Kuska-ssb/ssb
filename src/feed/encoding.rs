use super::error::Result;
use kuska_sodiumoxide::crypto::hash::sha256;
use serde_json::Value;

pub fn ssb_sha256(v: &Value) -> Result<sha256::Digest> {
    let v8encoding = stringify_json(&v)?
        .encode_utf16()
        .map(|ch_u16| (ch_u16 & 0xff) as u8)
        .collect::<Vec<u8>>();

    Ok(sha256::hash(&v8encoding[..]))
}

pub fn stringify_json(v: &Value) -> Result<String> {
    fn spaces(n: usize) -> &'static str {
        &"                                         "[..2 * n]
    }
    // see https://www.ecma-international.org/ecma-262/6.0/#sec-serializejsonobject
    fn append_json(buffer: &mut String, level: usize, v: &Value) -> Result<()> {
        match v {
            Value::Object(values) => {
                if values.is_empty() {
                    buffer.push_str("{}");
                } else {
                    buffer.push_str("{\n");
                    for (i, (key, value)) in values.iter().enumerate() {
                        buffer.push_str(spaces(level + 1));
                        buffer.push_str(&serde_json::to_string(&key)?);
                        buffer.push_str(": ");
                        append_json(buffer, level + 1, &value)?;
                        if i < values.len() - 1 {
                            buffer.push(',');
                        }
                        buffer.push('\n');
                    }
                    buffer.push_str(spaces(level));
                    buffer.push('}');
                }
            }
            Value::Array(values) => {
                if values.is_empty() {
                    buffer.push_str("[]");
                } else {
                    buffer.push_str("[\n");
                    for (i, value) in values.iter().enumerate() {
                        buffer.push_str(spaces(level + 1));
                        append_json(buffer, level + 1, &value)?;
                        if i < values.len() - 1 {
                            buffer.push(',');
                        }
                        buffer.push('\n');
                    }
                    buffer.push_str(spaces(level));
                    buffer.push(']');
                }
            }
            Value::String(value) => {
                buffer.push_str(&serde_json::to_string(&value)?);
            }
            Value::Number(value) => {
                let mut as_str = value.to_string();
                if as_str.contains('e') && !as_str.contains("e-") {
                    as_str = as_str.replace("e", "e+")
                }
                buffer.push_str(&as_str);
            }
            Value::Bool(value) => {
                buffer.push_str(if *value { "true" } else { "false" });
            }
            Value::Null => {
                buffer.push_str("null");
            }
        }
        Ok(())
    }
    let mut result = String::new();
    append_json(&mut result, 0, &v)?;
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    const JSON: &str = r#"{"a":0,"b":1.1,"c":null,"d":true,"f":false,"g":{},"h":{"h1":1},"i":[],"j":[1],"k":[1,2]}"#;
    #[test]
    fn test_json_stringify() -> Result<()> {
        let v: Value = serde_json::from_str(JSON)?;
        let json = stringify_json(&v)?;
        let expected = r#"{
  "a": 0,
  "b": 1.1,
  "c": null,
  "d": true,
  "f": false,
  "g": {},
  "h": {
    "h1": 1
  },
  "i": [],
  "j": [
    1
  ],
  "k": [
    1,
    2
  ]
}"#;

        assert_eq!(expected, json);
        Ok(())
    }
    #[test]
    fn test_verify_known_msg_integrity() -> Result<()> {
        let expected = "Cg0ZpZ8cV85G8UIIropgBOvM8+Srlv9LSGDNGnpdK44=";
        let message = r#"{"previous":"%seUEAo7PTyA7vNwnOrmGIsUFfpyRzOvzGVv1QCb/Fz8=.sha256","author":"@BIbVppzlrNiRJogxDYz3glUS7G4s4D4NiXiPEAEzxdE=.ed25519","sequence":37,"timestamp":1439392020612,"hash":"sha256","content":{"type":"post","text":"@paul real time replies didn't work.","repliesTo":"%xWKunF6nXD7XMC+D4cjwDMZWmBnmRu69w9T25iLNa1Q=.sha256","mentions":["%7UKRfZb2u8al4tYWHqM55R9xpE/KKVh9U0M6BdugGt4=.sha256"],"recps":[{"link":"@hxGxqPrplLjRG2vtjQL87abX4QKqeLgCwQpS730nNwE=.ed25519","name":"paul"}]},"signature":"gGxSPdBJZxp6x5f3HzQGoQSeSdh/C5AtymIn+miWa+lcC6DdqpRSgaeH9KHeLf+/CKhU6REYIpWaLr4CKDMfCg==.sig.ed25519"}"#;
        let message_value: Value = serde_json::from_str(&message)?;
        let current = base64::encode(&ssb_sha256(&message_value)?);
        assert_eq!(expected, current);
        Ok(())
    }

    #[test]
    fn test_msg_with_float_mantissa() -> Result<()> {
        let expected = "RUcldndjJUkEcZ5hX6zAj/xLlnh0n4BZ6ThJOW5RvIk=";
        let message = r#"{"previous":"%gbem82xZNVHbOM2pyOlxymsAfstdMFfGSoawWQtObX8=.sha256","author":"@TXKFQehlyoSn8UJAIVP/k2BjFINC591MlBC2e2d24mA=.ed25519","sequence":1557,"timestamp":1495245157893,"hash":"sha256","content":{"type":"post","transactionHash":9.691449834862513e+76,"address":7.073631810716965e+46,"event":"ActionAdded","text":"{\"actionID\":\"1\",\"amount\":\"0\",\"description\":\"Bind Ethereum events to Secure Scuttlebutt posts\"}}"},"signature":"/Qvm9ozEfl0Thyvs+mnwhLDReZ8xeKXA3hSXOxm53SFkLEnnJ+IF0l7LSqc56Y3vl8FwarJ6k0PGmcU3U8FMAw==.sig.ed25519"}"#;
        let message_value: Value = serde_json::from_str(&message)?;
        let current = base64::encode(&ssb_sha256(&message_value)?);
        assert_eq!(expected, current);
        Ok(())
    }

    #[test]
    fn test_msg_with_float_precision() -> Result<()> {
        let expected = "BUtTVIJyN5fUXzQy2uQfCCzlAg0s6laQQqFIu+kGnFM=";
        let message = r#"{"previous":"%ButTjV+H9VfONhX+lLbJb5LR+W14SFqbmjOfdMPZ5+4=.sha256","sequence":15034,"author":"@6ilZq3kN0F+dXFHAPjAwMm87JEb/VdB+LC9eIMW3sa0=.ed25519","timestamp":1567190273951.0159,"hash":"sha256","content":{"type":"vote","channel":null,"vote":{"link":"%GvtUsekEwsCj1cQ6+4Gihkm+ek99BhB537g1xUKjhsA=.sha256","value":1,"expression":"Like"}},"signature":"UkVfqDmBhHrDfMvFT8iUhEispAku/zbdXKCyRVlxYp2wNtJ4okwKE7hTkKhbiMVA7sGIV5dzHZyMotXCL46iDw==.sig.ed25519"}"#;
        let message_value: Value = serde_json::from_str(&message)?;
        let current = base64::encode(&ssb_sha256(&message_value)?);
        assert_eq!(expected, current);
        Ok(())
    }
}
