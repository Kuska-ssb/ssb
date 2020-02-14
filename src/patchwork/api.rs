use async_std::io::{Read, Write};
use serde_json;

use crate::rpc::{BodyType, RequestNo, RpcStream, RpcType};

use super::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorRes {
    pub name: String,
    pub message: String,
    pub stack: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoAmI {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestUserMessage {
    pub id: String,
    pub sequence: u64,
    pub ts: f64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHistoryStreamArgs {
    // id (FeedID, required): The id of the feed to fetch.
    pub id: String,

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

impl CreateHistoryStreamArgs {
    pub fn new(id: String) -> Self {
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

#[derive(Debug)]
pub enum ApiMethod {
    WhoAmI,
    Get,
    CreateHistoryStream,
    CreateFeedStream,
    Latest,
}

impl ApiMethod {
    pub fn selector(&self) -> &'static [&'static str] {
        use ApiMethod::*;
        match self {
            WhoAmI => &["whoami"],
            Get => &["get"],
            CreateHistoryStream => &["createHistoryStream"],
            CreateFeedStream => &["createFeedStream"],
            Latest => &["latest"],
        }
    }
    pub fn from_selector(s: &[&str]) -> Option<Self> {
        use ApiMethod::*;
        match s {
            ["whoami"] => Some(WhoAmI),
            ["get"] => Some(Get),
            ["createHistoryStream"] => Some(CreateHistoryStream),
            ["createFeedStream"] => Some(CreateFeedStream),
            ["latest"] => Some(Latest),
            _ => None,
        }
    }
}

pub struct ApiHelper<R: Read + Unpin, W: Write + Unpin> {
    rpc: RpcStream<R, W>,
}

impl<R: Read + Unpin, W: Write + Unpin> ApiHelper<R, W> {
    pub fn new(rpc: RpcStream<R, W>) -> Self {
        Self { rpc }
    }

    pub fn rpc(&mut self) -> &mut RpcStream<R, W> {
        &mut self.rpc
    }

    // whoami: sync
    // Get information about the current ssb-server user.
    pub async fn whoami_req_send(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(ApiMethod::WhoAmI.selector(), RpcType::Async, &args)
            .await?;
        Ok(req_no)
    }
    pub async fn whoami_res_send(&mut self, req_no: RequestNo, id: String) -> Result<()> {
        let body = serde_json::to_string(&WhoAmI { id })?;
        Ok(self
            .rpc
            .send_response(req_no, RpcType::Async, BodyType::JSON, body.as_bytes())
            .await?)
    }

    // get: async
    // Get a message by its hash-id. (sould start with %)
    pub async fn get_req_send(&mut self, msg_id: &str) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(ApiMethod::Get.selector(), RpcType::Async, &msg_id)
            .await?;
        Ok(req_no)
    }

    // createHistoryStream: source
    // (hist) Fetch messages from a specific user, ordered by sequence numbers.
    pub async fn create_history_stream_req_send(
        &mut self,
        args: &CreateHistoryStreamArgs,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::CreateHistoryStream.selector(),
                RpcType::Source,
                &args,
            )
            .await?;
        Ok(req_no)
    }
    pub async fn feed_res_send(&mut self, req_no: RequestNo, feed: &str) -> Result<()> {
        self.rpc
            .send_response(req_no, RpcType::Async, BodyType::JSON, feed.as_bytes())
            .await?;
        Ok(())
    }

    // createFeedStream: source
    // (feed) Fetch messages ordered by their claimed timestamps.
    pub async fn send_create_feed_stream<'a>(
        &mut self,
        args: &CreateStreamArgs<u64>,
    ) -> Result<RequestNo> {
        let req_no = self
            .rpc
            .send_request(
                ApiMethod::CreateFeedStream.selector(),
                RpcType::Source,
                &args,
            )
            .await?;
        Ok(req_no)
    }

    // latest: source
    // Get the seq numbers of the latest messages of all users in the database.
    pub async fn send_latest(&mut self) -> Result<RequestNo> {
        let args: [&str; 0] = [];
        let req_no = self
            .rpc
            .send_request(ApiMethod::Latest.selector(), RpcType::Async, &args)
            .await?;
        Ok(req_no)
    }
}
