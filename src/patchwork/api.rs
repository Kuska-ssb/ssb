use async_std::io::{Read, Write};
use serde_json;
use std::str::FromStr;

use crate::rpc::{RpcClient, Header, RequestNo, RpcType};
use crate::feed::Message;
use crate::feed::Feed;

use super::error::{Error,Result};

#[derive(Debug, Deserialize)]
pub struct ErrorRes {
    pub name: String,
    pub message: String,
    pub stack: String,
}

#[derive(Debug, Deserialize)]
pub struct WhoAmI {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct LatestUserMessage {
    pub id: String,
    pub sequence: u64,
    pub ts: u64,
}

// https://github.com/ssbc/ssb-db/blob/master/api.md
#[derive(Debug, Serialize)]
pub struct CreateStreamArgs<K> {
    /// live (boolean, default: false): Keep the stream open and emit new messages as they are received
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,

    /// gt (greater than), gte (greater than or equal) define the lower bound of the range to be streamed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<K>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<K>,

    /// lt (less than), lte (less than or equal) define the higher bound of the range to be streamed. Only key/value pairs where the key is less than (or equal to) this option will be included in the range. When reverse=true the order will be reversed, but the records streamed will be the same.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<K>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<K>,

    /// reverse (boolean, default: false): a boolean, set true and the stream output will be reversed. Beware that due to the way LevelDB works, a reverse seek will be slower than a forward seek.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,

    /// keys (boolean, default: true): whether the data event should contain keys. If set to true and values set to false then data events will simply be keys, rather than objects with a key property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<bool>,

    /// values (boolean, default: true): whether the data event should contain values. If set to true and keys set to false then data events will simply be values, rather than objects with a value property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<bool>,

    /// limit (number, default: -1): limit the number of results collected by this stream. This number represents a maximum number of results and may not be reached if you get to the end of the data first. A value of -1 means there is no limit. When reverse=true the highest keys will be returned instead of the lowest keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// fillCache (boolean, default: false): wheather LevelDB's LRU-cache should be filled with data read.
    #[serde(rename = "fillCache")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_cache: Option<bool>,
    /// keyEncoding / valueEncoding (string): the encoding applied to each read piece of data.
    #[serde(rename = "keyEncoding")]
    pub key_encoding: Option<String>,

    #[serde(rename = "valueEncoding")]
    pub value_encoding: Option<String>,
}

impl<K> Default for CreateStreamArgs<K> {
    fn default() -> Self {
        Self {
            live: None,
            gt: None,
            gte: None,
            lt: None,
            lte: None,
            reverse: None,
            keys: None,
            values: None,
            limit: None,
            fill_cache: None,
            key_encoding: None,
            value_encoding: None,
        }
    }
}

impl<K> CreateStreamArgs<K> {
    pub fn live(self: Self, live: bool) -> Self {
        Self {
            live: Some(live),
            ..self
        }
    }
    pub fn gt(self: Self, v: K) -> Self {
        Self {
            gt: Some(v),
            ..self
        }
    }
    pub fn gte(self: Self, v: K) -> Self {
        Self {
            gte: Some(v),
            ..self
        }
    }
    pub fn lt(self: Self, v: K) -> Self {
        Self {
            lt: Some(v),
            ..self
        }
    }
    pub fn lte(self: Self, v: K) -> Self {
        Self {
            lte: Some(v),
            ..self
        }
    }
    pub fn reverse(self: Self, reversed: bool) -> Self {
        Self {
            reverse: Some(reversed),
            ..self
        }
    }
    pub fn keys_values(self: Self, keys: bool, values: bool) -> Self {
        Self {
            keys: Some(keys),
            values: Some(values),
            ..self
        }
    }
    pub fn encoding(self: Self, keys: String, values: String) -> Self {
        Self {
            key_encoding: Some(keys),
            value_encoding: Some(values),
            ..self
        }
    }
    pub fn limit(self: Self, limit: u64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateHistoryStreamArgs<'a> {
    // id (FeedID, required): The id of the feed to fetch.
    pub id: &'a str,

    /// (number, default: 0): If seq > 0, then only stream messages with sequence numbers greater than seq.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,

    /// live (boolean, default: false): Keep the stream open and emit new messages as they are received
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<bool>,
    /// keys (boolean, default: true): whether the data event should contain keys. If set to true and values set to false then data events will simply be keys, rather than objects with a key property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<bool>,

    /// values (boolean, default: true): whether the data event should contain values. If set to true and keys set to false then data events will simply be values, rather than objects with a value property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<bool>,

    /// limit (number, default: -1): limit the number of results collected by this stream. This number represents a maximum number of results and may not be reached if you get to the end of the data first. A value of -1 means there is no limit. When reverse=true the highest keys will be returned instead of the lowest keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

impl<'a> CreateHistoryStreamArgs<'a> {
    pub fn new(id: &'a str) -> Self {
        Self {
            id,
            seq: None,
            live: None,
            keys: None,
            values: None,
            limit: None,
        }
    }
    pub fn starting_seq(self: Self, seq: u64) -> Self {
        Self {
            seq: Some(seq),
            ..self
        }
    }
    pub fn live(self: Self, live: bool) -> Self {
        Self {
            live: Some(live),
            ..self
        }
    }
    pub fn keys_values(self: Self, keys: bool, values: bool) -> Self {
        Self {
            keys: Some(keys),
            values: Some(values),
            ..self
        }
    }
    pub fn limit(self: Self, limit: u64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }
}


fn parse_json<'a, T: serde::Deserialize<'a>>(
    header: &'a Header,
    body: &'a [u8],
) -> Result<T> {
    if header.is_end_or_error {
        let error: ErrorRes = serde_json::from_slice(&body[..])?;
        Err(Error::ServerMessage(format!("{:?}", error)))
    } else {
        let res: T = serde_json::from_slice(&body[..])?;
        Ok(res)
    }
}

pub fn parse_whoami(header: &Header, body: &[u8]) -> Result<WhoAmI> {
    parse_json::<WhoAmI>(&header, body)
}

pub fn parse_message(header: &Header, body: &[u8]) -> Result<Message> {
    if header.is_end_or_error {
        let error: ErrorRes = serde_json::from_slice(&body[..])?;
        Err(Error::ServerMessage(format!("{:?}", error)))
    } else {
        Ok(Message::from_slice(body)?)
    }
}

pub fn parse_feed(header: &Header, body: &[u8]) -> Result<Feed> {
    if header.is_end_or_error {
        let error: ErrorRes = serde_json::from_slice(&body[..])?;
        Err(Error::ServerMessage(format!("{:?}", error)))
    } else {
        Ok(Feed::from_str(&String::from_utf8_lossy(&body))?)
    }
}

pub fn parse_latest(header: &Header, body: &[u8]) -> Result<LatestUserMessage> {
    parse_json::<LatestUserMessage>(&header, body)
}

pub struct ApiClient<R: Read + Unpin, W: Write + Unpin> {
    rpc: RpcClient<R, W>,
}

impl<R: Read + Unpin, W: Write + Unpin> ApiClient<R, W> {
    pub fn new(rpc: RpcClient<R, W>) -> Self {
        Self { rpc }
    }

    pub fn rpc(&mut self) -> &mut RpcClient<R, W> {
        &mut self.rpc
    }

    // whoami: sync
    // Get information about the current ssb-server user.
    pub async fn send_whoami(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self.rpc.send(&["whoami"], RpcType::Async, &args).await?;
        Ok(req_no)
    }

    // get: async
    // Get a message by its hash-id. (sould start with %)
    pub async fn send_get(&mut self, msg_id: &str) -> Result<RequestNo> {
        let req_no = self.rpc.send(&["get"], RpcType::Async, &msg_id).await?;
        Ok(req_no)
    }
    // createHistoryStream: source
    // (hist) Fetch messages from a specific user, ordered by sequence numbers.
    pub async fn send_create_history_stream<'a>(
        &mut self,
        args: &'a CreateHistoryStreamArgs<'a>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send(&["createHistoryStream"], RpcType::Source, &args)
            .await?;
        Ok(req_no)
    }

    // createFeedStream: source
    // (feed) Fetch messages ordered by their claimed timestamps.
    pub async fn send_create_feed_stream<'a>(
        &mut self,
        args: &'a CreateStreamArgs<u64>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send(&["createFeedStream"], RpcType::Source, &args)
            .await?;
        Ok(req_no)
    }

    // latest: source
    // Get the seq numbers of the latest messages of all users in the database.
    pub async fn send_latest(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self.rpc.send(&["latest"], RpcType::Source, &args).await?;
        Ok(req_no)
    }
}

