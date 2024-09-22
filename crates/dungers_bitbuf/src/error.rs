#[derive(thiserror::Error, Debug)]
#[error("was about to overrun a buffer")]
pub struct OverflowError;

#[derive(thiserror::Error, Debug)]
pub enum ReadIntoBufferError {
    #[error(transparent)]
    Overflow(#[from] OverflowError),
    #[error("buffer too small")]
    BufferTooSmall,
}

#[cfg(feature = "varint")]
#[derive(thiserror::Error, Debug)]
pub enum ReadVarintError {
    #[error(transparent)]
    Overflow(#[from] OverflowError),
    #[error("malformed varint")]
    MalformedVarint,
}
