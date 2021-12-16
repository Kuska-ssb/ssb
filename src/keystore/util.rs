use std::string::ToString;

pub fn to_io_error<T: ToString>(err: T) -> async_std::io::Error {
    async_std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
}
