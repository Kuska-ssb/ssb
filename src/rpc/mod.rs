mod client;
mod error;

pub use client::{RpcStream,RecvMsg, RequestNo, RpcType, Body, BodyRef};
pub use error::{Error,Result};
