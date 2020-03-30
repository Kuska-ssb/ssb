use super::error::{Error, Result};

use async_std::io;
use async_std::prelude::*;
use log::debug;

use kuska_handshake::async_std::{BoxStreamRead, BoxStreamWrite};

pub type RequestNo = i32;

const HEADER_SIZE: usize = 9;

const RPC_HEADER_STREAM_FLAG: u8 = 1 << 3;
const RPC_HEADER_END_OR_ERROR_FLAG: u8 = 1 << 2;
const RPC_HEADER_BODY_TYPE_MASK: u8 = 0b11;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BodyType {
    Binary,
    UTF8,
    JSON,
}

#[derive(Deserialize)]
pub struct Body {
    pub name: Vec<String>,
    #[serde(rename = "type")]
    pub rpc_type: RpcType,
    pub args: serde_json::Value,
}

#[derive(Serialize)]
pub struct BodyRef<'a, T: serde::Serialize> {
    pub name: &'a [&'a str],
    #[serde(rename = "type")]
    pub rpc_type: RpcType,
    pub args: &'a T,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum RpcType {
    #[serde(rename = "async")]
    Async,
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "duplex")]
    Duplex,
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub req_no: RequestNo,
    pub is_stream: bool,
    pub is_end_or_error: bool,
    pub body_type: BodyType,
    pub body_len: u32,
}

#[derive(Serialize, Deserialize)]
struct ErrorMessage<'a> {
    name: &'a str,
    stack: &'a str,
    message: &'a str,
}

impl Header {
    pub fn from_slice(bytes: &[u8]) -> Result<Header> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::HeaderSizeTooSmall);
        }

        let is_stream = (bytes[0] & RPC_HEADER_STREAM_FLAG) == RPC_HEADER_STREAM_FLAG;
        let is_end_or_error =
            (bytes[0] & RPC_HEADER_END_OR_ERROR_FLAG) == RPC_HEADER_END_OR_ERROR_FLAG;
        let body_type = match bytes[0] & RPC_HEADER_BODY_TYPE_MASK {
            0 => BodyType::Binary,
            1 => BodyType::UTF8,
            2 => BodyType::JSON,
            _ => return Err(Error::InvalidBodyType),
        };

        let mut body_len_buff = [0u8; 4];
        body_len_buff.copy_from_slice(&bytes[1..5]);
        let body_len = u32::from_be_bytes(body_len_buff);

        let mut reqno_buff = [0u8; 4];
        reqno_buff.copy_from_slice(&bytes[5..9]);
        let req_no = i32::from_be_bytes(reqno_buff);

        Ok(Header {
            req_no,
            is_stream,
            is_end_or_error,
            body_type,
            body_len,
        })
    }

    pub fn to_array(&self) -> [u8; 9] {
        let mut flags: u8 = 0;
        if self.is_end_or_error {
            flags |= RPC_HEADER_END_OR_ERROR_FLAG;
        }
        if self.is_stream {
            flags |= RPC_HEADER_STREAM_FLAG;
        }
        flags |= match self.body_type {
            BodyType::Binary => 0,
            BodyType::UTF8 => 1,
            BodyType::JSON => 2,
        };
        let len = self.body_len.to_be_bytes();
        let req_no = self.req_no.to_be_bytes();

        let mut encoded = [0u8; 9];
        encoded[0] = flags;
        encoded[1..5].copy_from_slice(&len[..]);
        encoded[5..9].copy_from_slice(&req_no[..]);

        encoded
    }
}

pub struct RpcStream<R: io::Read + Unpin, W: io::Write + Unpin> {
    box_reader: BoxStreamRead<R>,
    box_writer: BoxStreamWrite<W>,
    req_no: RequestNo,
}

pub enum RecvMsg {
    RpcRequest(Body),
    RpcResponse(BodyType, Vec<u8>),
    OtherRequest(BodyType, Vec<u8>),
    ErrorResponse(String),
    CancelStreamRespose(),
}

impl<R: io::Read + Unpin, W: io::Write + Unpin> RpcStream<R, W> {
    pub fn new(box_reader: BoxStreamRead<R>, box_writer: BoxStreamWrite<W>) -> RpcStream<R, W> {
        RpcStream {
            box_reader,
            box_writer,
            req_no: 0,
        }
    }

    pub async fn recv(&mut self) -> Result<(RequestNo, RecvMsg)> {
        let mut rpc_header_raw = [0u8; 9];
        self.box_reader.read_exact(&mut rpc_header_raw[..]).await?;
        let rpc_header = Header::from_slice(&rpc_header_raw[..])?;

        let mut body_raw: Vec<u8> = vec![0; rpc_header.body_len as usize];
        self.box_reader.read_exact(&mut body_raw[..]).await?;

        debug!(
            "rpc-recv {:?} '{}'",
            rpc_header,
            String::from_utf8_lossy(&body_raw[..])
        );

        if rpc_header.req_no > 0 {
            match serde_json::from_slice(&body_raw) {
                Ok(rpc_body) => Ok((rpc_header.req_no, RecvMsg::RpcRequest(rpc_body))),
                Err(_) => Ok((
                    rpc_header.req_no,
                    RecvMsg::OtherRequest(rpc_header.body_type, body_raw),
                )),
            }
        } else if rpc_header.is_end_or_error {
            if rpc_header.is_stream {
                Ok((-rpc_header.req_no, RecvMsg::CancelStreamRespose()))
            } else {
                let err: ErrorMessage = serde_json::from_slice(&body_raw)?;
                Ok((
                    -rpc_header.req_no,
                    RecvMsg::ErrorResponse(err.message.to_string()),
                ))
            }
        } else {
            Ok((
                -rpc_header.req_no,
                RecvMsg::RpcResponse(rpc_header.body_type, body_raw),
            ))
        }
    }

    pub async fn send_request<T: serde::Serialize>(
        &mut self,
        name: &[&str],
        rpc_type: RpcType,
        args: &T,
    ) -> Result<RequestNo> {
        self.req_no += 1;

        let body_str = serde_json::to_string(&BodyRef {
            name,
            rpc_type,
            args: &[&args],
        })?;

        let rpc_header = Header {
            req_no: self.req_no,
            is_stream: rpc_type == RpcType::Source,
            is_end_or_error: false,
            body_type: BodyType::JSON,
            body_len: body_str.as_bytes().len() as u32,
        };

        debug!("rpc-send {:?} '{}'", rpc_header, body_str);

        self.box_writer
            .write_all(&rpc_header.to_array()[..])
            .await?;
        self.box_writer.write_all(body_str.as_bytes()).await?;
        self.box_writer.flush().await?;

        Ok(self.req_no)
    }

    pub async fn send_response(
        &mut self,
        req_no: RequestNo,
        rpc_type: RpcType,
        body_type: BodyType,
        body: &[u8],
    ) -> Result<()> {
        let rpc_header = Header {
            req_no: -req_no,
            is_stream: rpc_type == RpcType::Source,
            is_end_or_error: false,
            body_type,
            body_len: body.len() as u32,
        };

        debug!(
            "rpc-send {:?} '{}'",
            rpc_header,
            String::from_utf8_lossy(body)
        );

        self.box_writer
            .write_all(&rpc_header.to_array()[..])
            .await?;
        self.box_writer.write_all(body).await?;
        self.box_writer.flush().await?;

        Ok(())
    }

    pub async fn send_error(
        &mut self,
        req_no: RequestNo,
        rpc_type: RpcType,
        message: &str,
    ) -> Result<()> {
        let body_bytes = serde_json::to_string(&ErrorMessage {
            name: "Error",
            stack: "",
            message,
        })?;

        let is_stream = match rpc_type {
            RpcType::Async => false,
            _ => true,
        };

        let rpc_header = Header {
            req_no: -req_no,
            is_stream,
            is_end_or_error: true,
            body_type: BodyType::UTF8,
            body_len: body_bytes.as_bytes().len() as u32,
        };

        debug!("rpc-send {:?} '{}'", rpc_header, body_bytes);

        self.box_writer
            .write_all(&rpc_header.to_array()[..])
            .await?;
        self.box_writer.write_all(body_bytes.as_bytes()).await?;
        self.box_writer.flush().await?;

        Ok(())
    }

    pub async fn send_stream_eof(&mut self, req_no: RequestNo) -> Result<()> {
        let body_bytes = b"true";

        let rpc_header = Header {
            req_no: -req_no,
            is_stream: true,
            is_end_or_error: true,
            body_type: BodyType::JSON,
            body_len: body_bytes.len() as u32,
        };

        debug!(
            "rpc-send {:?} '{}'",
            rpc_header,
            String::from_utf8_lossy(body_bytes)
        );

        self.box_writer
            .write_all(&rpc_header.to_array()[..])
            .await?;
        self.box_writer.write_all(&body_bytes[..]).await?;
        self.box_writer.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.box_writer.goodbye().await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{BodyType, Header};

    #[test]
    fn test_header_encoding_1() {
        let h = Header::from_slice(
            &(Header {
                req_no: 5,
                is_stream: true,
                is_end_or_error: false,
                body_type: BodyType::JSON,
                body_len: 123,
            }
            .to_array())[..],
        )
        .unwrap();
        assert_eq!(h.req_no, 5);
        assert_eq!(h.is_stream, true);
        assert_eq!(h.is_end_or_error, false);
        assert_eq!(h.body_type, BodyType::JSON);
        assert_eq!(h.body_len, 123);
    }

    #[test]
    fn test_header_encoding_2() {
        let h = Header::from_slice(
            &(Header {
                req_no: -5,
                is_stream: false,
                is_end_or_error: true,
                body_type: BodyType::Binary,
                body_len: 2123,
            }
            .to_array())[..],
        )
        .unwrap();
        assert_eq!(h.req_no, -5);
        assert_eq!(h.is_stream, false);
        assert_eq!(h.is_end_or_error, true);
        assert_eq!(h.body_type, BodyType::Binary);
        assert_eq!(h.body_len, 2123);
    }
}
