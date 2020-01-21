use sodiumoxide::crypto::auth;

pub const SSB_NET_ID: &str = "d4a1cb88a66f02f8db635ce26441cc5dac1b08420ceaac230839b755845a9ffb";
pub fn ssb_net_id() -> auth::Key {
    auth::Key::from_slice(&hex::decode(SSB_NET_ID).unwrap()).unwrap()
}
