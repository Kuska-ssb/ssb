use sodiumoxide::crypto::sign::ed25519;
use async_std::io;
use crate::util::to_ioerr;
use crate::crypto::ToSodiumObject;

pub struct Invite {
    pub domain : String,
    pub port : u16,
    pub pub_pk: ed25519::PublicKey,
    pub invite_sk: ed25519::SecretKey,
}

impl Invite {
    pub fn from_code(code : &str) -> Result<Self,io::Error> {
        let domain_port_keys : Vec<_> = code.split(":").collect();
        if domain_port_keys.len() != 3 {
            return Err(to_ioerr("invalid code"));
        }

        let domain = domain_port_keys[0].to_string();
        let port = domain_port_keys[1].parse::<u16>().map_err(to_ioerr)?;
        let pk_sk :Vec<_> = domain_port_keys[2].split("~").collect();

        if pk_sk.len() != 2 {
            return Err(to_ioerr("invalid code keys"));
        }
        let pub_pk = pk_sk[0][1..].to_ed25519_pk()?;
        let invite_sk = pk_sk[1][..].to_ed25519_sk_no_suffix()?;

        Ok(Invite { domain, port, pub_pk, invite_sk })
    }

}

#[cfg(test)]
mod test {
    const picopub : &str = "ssb-pub.picodevelopment.nl:8008:@UFDjYpDN89OTdow4sqZP5eEGGcy+1eN/HNc5DMdMI0M=.ed25519~ibtGafFt7myC9yEyJ6Oq7gWuS2+2ue9XI3iyE9QXSwI=";
}
