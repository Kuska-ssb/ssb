#[derive(Debug, Serialize, Deserialize)]
pub struct TanglesThread {
    /// id (string, required): The key of the root message of a thread, for
    /// which replies are to be fetched and returned.
    pub root: String,

    /// keys (boolean, default: false): whether the data event should contain
    /// keys. If set to true and values set to false then data events will
    /// simply be keys, rather than objects with a key property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<bool>,

    /// values (boolean, default: true): whether the data event should contain
    /// values. If set to true and keys set to false then data events will
    /// simply be values, rather than objects with a value property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<bool>,

    /// limit (number, default: -1): limit the number of results collected by
    /// this stream. This number represents a maximum number of results and may
    /// not be reached if you get to the end of the data first. A value of -1
    /// means there is no limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// private (boolean, default: false): attempt to unbox the encrypted
    /// messages comprising the thread. This should be set to true if the
    /// tangle messages are private.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
}

impl TanglesThread {
    pub fn new(root: String) -> Self {
        Self {
            root,
            keys: None,
            values: None,
            limit: None,
            private: None,
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

    pub fn private(self, private: bool) -> Self {
        Self {
            private: Some(private),
            ..self
        }
    }
}
