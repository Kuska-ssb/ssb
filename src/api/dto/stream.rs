// https://github.com/ssbc/ssb-db/blob/master/api.md
#[derive(Debug, Serialize)]
pub struct CreateStreamIn<K> {
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
    pub limit: Option<i64>,

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

impl<K> Default for CreateStreamIn<K> {
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

impl<K> CreateStreamIn<K> {
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
    pub fn limit(self: Self, limit: i64) -> Self {
        Self {
            limit: Some(limit),
            ..self
        }
    }
}
