#![allow(clippy::single_match)]

use get_if_addrs::{get_if_addrs, IfAddr};

use log::warn;
use std::string::ToString;

use async_std::net::{IpAddr, SocketAddr, UdpSocket};

use once_cell::sync::Lazy;
use regex::Regex;

use crate::{crypto::ed25519, crypto::ToSodiumObject};

use super::error::{Error, Result};

pub static BROADCAST_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"net:([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+):([0-9]+)~shs:([0-9a-zA-Z=/]+)").unwrap()
});

pub struct LanBroadcast {
    destination: String,
    packets: Vec<(SocketAddr, SocketAddr, String)>,
}

impl LanBroadcast {
    pub async fn new(id: &ed25519::PublicKey, rpc_port: u16) -> Result<Self> {
        let server_pk = base64::encode(&id);

        let mut packets = Vec::new();

        for if_addr in get_if_addrs()? {
            let addrs = match if_addr.addr {
                IfAddr::V4(v4) if !v4.is_loopback() && v4.broadcast.is_some() => {
                    Some((IpAddr::V4(v4.ip), IpAddr::V4(v4.broadcast.unwrap())))
                }
                IfAddr::V6(v6) if !v6.is_loopback() && v6.broadcast.is_some() => {
                    Some((IpAddr::V6(v6.ip), IpAddr::V6(v6.broadcast.unwrap())))
                }
                _ => None,
            };

            if let Some((local, broadcast)) = addrs {
                let local_addr = SocketAddr::new(local, rpc_port);
                let broadcast_addr = SocketAddr::new(broadcast, rpc_port);
                let msg = format!("net:{}:{}~shs:{}", local, rpc_port, server_pk);
                match UdpSocket::bind(SocketAddr::new(local, rpc_port)).await {
                    Ok(_) => packets.push((local_addr, broadcast_addr, msg)),
                    Err(err) => warn!("cannot broadcast to {:?} {:?}", local_addr, err),
                };
            }
        }
        let destination = format!("255.255.255.255:{}", rpc_port);
        Ok(LanBroadcast {
            packets,
            destination,
        })
    }
    pub async fn send(&self) {
        for msg in &self.packets {
            if let Ok(socket) = UdpSocket::bind(msg.0).await {
                let _ = socket.set_broadcast(true);
                match socket.send_to(msg.2.as_bytes(), &self.destination).await {
                    Err(err) => warn!(target:"solar", "Error broadcasting {}",err),
                    _ => {}
                }
            }
        }
    }

    pub fn parse(msg: &str) -> Option<(String, u32, ed25519::PublicKey)> {
        let parse_shs = |addr: &str| -> Result<_> {
            let captures = BROADCAST_REGEX
                .captures(&addr)
                .ok_or(Error::InvalidBroadcastMessage)?;

            let ip = captures[1].to_string();
            let port = captures[2].parse::<u32>()?;
            let server_pk = captures[3].to_ed25519_pk_no_suffix()?;

            Ok((ip, port, server_pk))
        };

        for addr in msg.split(';') {
            if let Ok(shs) = parse_shs(addr) {
                return Some(shs);
            }
        }

        None
    }
}
