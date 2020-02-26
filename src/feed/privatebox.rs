use crate::crypto::ToSodiumObject;

use sodiumoxide::crypto::sign::SecretKey;
use sodiumoxide::crypto::{scalarmult::curve25519, secretbox, sign::ed25519};

use super::error::{Error, Result};

pub const SUFFIX: &str = ".box";

pub const MAX_RECIPIENTS: u8 = 7;

const RECIPIENT_COUNT_LEN: usize = 1;
const ENCRYPTED_HEADER_LEN: usize = RECIPIENT_COUNT_LEN + secretbox::KEYBYTES + secretbox::MACBYTES;

pub fn is_privatebox(text: &str) -> bool {
    text.ends_with(SUFFIX)
}

pub fn privatebox_cipher(plaintext: &str, recipients: &[&str]) -> Result<String> {
    let recipients: crate::crypto::Result<Vec<_>> = recipients
        .iter()
        .map(|id| id[1..].to_ed25519_pk())
        .collect();

    let recipients = recipients?;

    let recipients_ref: Vec<_> = recipients.iter().map(|r| r).collect();

    let ciphertext = cipher(plaintext.as_bytes(), &recipients_ref[..])?;

    Ok(format!("{}{}", base64::encode(&ciphertext), SUFFIX))
}

pub fn privatebox_decipher(ciphertext: &str, sk: &SecretKey) -> Result<Option<String>> {
    let msg = &ciphertext.as_bytes()[..ciphertext.len() - SUFFIX.len()];
    let msg = base64::decode(msg)?;

    let plaintext = decipher(&msg, &sk)?.map(|msg| String::from_utf8_lossy(&msg).to_string());

    Ok(plaintext)
}

fn cipher(plaintext: &[u8], recipients: &[&ed25519::PublicKey]) -> Result<Box<[u8]>> {
    // Precondition checks
    if plaintext.is_empty() {
        return Err(Error::EmptyPlaintext);
    }

    if recipients.is_empty() || recipients.len() > MAX_RECIPIENTS as usize {
        return Err(Error::BadRecipientCount);
    }

    // Generated Curve25519 key pair to encrypt message header
    let (h_pk, h_sk) = ed25519::gen_keypair();

    // Generated random 32-byte secret key used to encrypt the message body
    let y = sodiumoxide::crypto::secretbox::gen_key();

    // Encrypt the plaintext with y, with a random nonce
    let nonce = secretbox::gen_nonce();
    let cipher_message = secretbox::seal(plaintext, &nonce, &y);

    // The sender uses scalar multiplication to derive a shared secret for each recipient,
    //   and encrypts (number_of_recipents || y) for each one
    let h_sk_scalar = &h_sk.to_curve25519();
    let mut plain_header = [0u8; RECIPIENT_COUNT_LEN + secretbox::KEYBYTES];
    plain_header[0] = recipients.len() as u8;
    plain_header[RECIPIENT_COUNT_LEN..].copy_from_slice(&y[..]);

    let mut buffer: Vec<u8> = Vec::with_capacity(
        secretbox::NONCEBYTES
            + ed25519::PUBLICKEYBYTES
            + ENCRYPTED_HEADER_LEN * recipients.len()
            + secretbox::MACBYTES
            + plaintext.len(),
    );

    buffer.extend_from_slice(&nonce[..]);
    buffer.extend_from_slice(&h_pk.to_curve25519()[..]);

    for recipient in recipients {
        let key = curve25519::scalarmult(&h_sk_scalar, &recipient.to_curve25519())
            .map_err(|_| Error::CryptoScalarMultFailed)?;

        let key = secretbox::Key::from_slice(&key[..]).ok_or(Error::CryptoKeyFromGrupFailed)?;

        buffer.extend_from_slice(&secretbox::seal(&plain_header[..], &nonce, &key));
    }
    buffer.extend_from_slice(&cipher_message[..]);

    Ok(buffer.into_boxed_slice())
}

fn decipher(ciphertext: &[u8], sk: &SecretKey) -> Result<Option<Vec<u8>>> {
    let mut cursor = ciphertext;

    let nonce = secretbox::Nonce::from_slice(&cursor[..secretbox::NONCEBYTES])
        .ok_or(Error::CannotReadNonce)?;
    cursor = &cursor[secretbox::NONCEBYTES..];

    let h_pk = curve25519::GroupElement::from_slice(&cursor[..ed25519::PUBLICKEYBYTES])
        .ok_or(Error::CannotReadNonce)?;
    cursor = &cursor[ed25519::PUBLICKEYBYTES..];

    let key = curve25519::scalarmult(&sk.to_curve25519(), &h_pk)
        .and_then(|key| secretbox::Key::from_slice(&key[..]).ok_or(()))
        .map_err(|_| Error::CannotCreateKey)?;

    let mut header_no = 0;
    while header_no < MAX_RECIPIENTS && cursor.len() > ENCRYPTED_HEADER_LEN + secretbox::MACBYTES {
        if let Ok(header) = secretbox::open(&cursor[..ENCRYPTED_HEADER_LEN], &nonce, &key) {
            let encrypted_message_offset = ENCRYPTED_HEADER_LEN * (header[0] - header_no) as usize;
            let y = secretbox::Key::from_slice(&header[1..]).ok_or(Error::CannotCreateKey)?;
            let plaintext = secretbox::open(&cursor[encrypted_message_offset..], &nonce, &y)
                .map_err(|_| Error::FailedToDecipher)?;
            return Ok(Some(plaintext));
        }
        header_no += 1;
        cursor = &cursor[49..];
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::keystore::OwnedIdentity;

    #[test]
    fn test_msg_cipher_to_one() -> Result<()> {
        let (u1_pk, u1_sk) = ed25519::gen_keypair();
        let plaintext = "hola".as_bytes();
        let ciphertext = cipher(plaintext, &[&u1_pk])?;
        let plaintext_1 = decipher(&ciphertext, &u1_sk)?.unwrap();
        assert_eq!(plaintext.to_vec(), plaintext_1);
        Ok(())
    }

    #[test]
    fn test_msg_cipher_to_one_helper() -> Result<()> {
        let id = OwnedIdentity::new();
        let plaintext = "holar";
        let ciphertext = privatebox_cipher(plaintext, &[&id.id])?;
        assert_eq!(is_privatebox(&ciphertext), true);
        let plaintext_1 = privatebox_decipher(&ciphertext, &id.sk)?.unwrap();
        assert_eq!(plaintext, plaintext_1);
        Ok(())
    }

    #[test]
    fn test_msg_cipher_to_none() -> Result<()> {
        let (u1_pk, _) = ed25519::gen_keypair();
        let (_, u1_sk) = ed25519::gen_keypair();
        let plaintext = "hola".as_bytes();
        let ciphertext = cipher(plaintext, &[&u1_pk])?;
        let plaintext_1 = decipher(&ciphertext, &u1_sk)?;
        assert_eq!(None, plaintext_1);
        Ok(())
    }

    #[test]
    fn test_msg_cipher_to_multiple() -> Result<()> {
        let u = (0..7).map(|_| ed25519::gen_keypair()).collect::<Vec<_>>();
        let u_pk = u.iter().map(|(pk, _)| pk).collect::<Vec<_>>();

        let plaintext = "hola".as_bytes();
        let ciphertext = cipher(plaintext, &u_pk)?;

        for (_, sk) in u.iter() {
            let plaintext_1 = decipher(&ciphertext, sk)?.unwrap();
            assert_eq!(plaintext.to_vec(), plaintext_1);
        }

        Ok(())
    }
}
