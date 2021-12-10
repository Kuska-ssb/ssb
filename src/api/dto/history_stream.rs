#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHistoryStreamIn {
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
    pub limit: Option<i64>,
}

impl CreateHistoryStreamIn {
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
    pub fn after_seq(self, seq: u64) -> Self {
        Self {
            seq: Some(seq),
            ..self
        }
    }
    pub fn live(self, live: bool) -> Self {
        Self {
            live: Some(live),
            ..self
        }
    }
    pub fn keys_values(self, keys: bool, values: bool) -> Self {
        Self {
            keys: Some(keys),
            values: Some(values),
            ..self
        }
    }
    pub fn limit(self, limit: i64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }
}
