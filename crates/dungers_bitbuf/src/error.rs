#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("was about to overrun a buffer")]
    Overflow,
    #[cfg(feature = "varint")]
    #[error("malformed varint")]
    MalformedVarint,
}

pub type Result<T> = std::result::Result<T, Error>;
