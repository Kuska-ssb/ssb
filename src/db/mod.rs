mod error;
mod feeds;

pub use error::{Error, Result};
pub use feeds::{FeedStorageIterator, FeedStorageReverseIterator, FeedsStorage};
