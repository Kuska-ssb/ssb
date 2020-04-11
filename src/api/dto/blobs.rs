#[derive(Debug, Serialize, Deserialize)]
pub struct BlobsGetIn {
    // key : ID of the blob. Required.
    pub key: String,

    // size : Expected size of the blob in bytes.
    // If the blob is not exactly this size then reject the request. Optional.
    pub size: Option<u64>,

    // max 	Maximum size of the blob in bytes. If the blob is larger then reject
    // the request. Only makes sense to specify max if you don’t already know size. Optional.
    pub max: Option<u64>,
}

impl BlobsGetIn {
    pub fn new(key: String) -> Self {
        Self {
            key,
            size: None,
            max: None,
        }
    }
    pub fn size(self: Self, size: u64) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
    pub fn max(self: Self, max: u64) -> Self {
        Self {
            max: Some(max),
            ..self
        }
    }
}
