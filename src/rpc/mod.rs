mod client;
mod error;

pub use client::{RpcClient,Header, RequestNo, RpcType, Body};
pub use error::{Error,Result};
