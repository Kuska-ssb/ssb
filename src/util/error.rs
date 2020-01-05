use async_std::io;
use std::fmt::Debug;

pub fn to_ioerr<T: Debug>(err: T) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("{:?}",err))
}

