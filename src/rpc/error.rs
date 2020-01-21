#[derive(Debug)]
pub enum Error {
    HeaderSizeTooSmall,
    InvalidBodyType,
    Io(async_std::io::Error),
    Json(serde_json::Error),
}

impl From<async_std::io::Error> for Error {
    fn from(err: async_std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
