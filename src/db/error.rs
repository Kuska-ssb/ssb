#[derive(Debug)]
pub enum Error {
    InvalidSequenceNo,
    MismatchReadingSecondSize,
    CompressionError(String),
    Utf8(std::string::FromUtf8Error),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::Utf8(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
