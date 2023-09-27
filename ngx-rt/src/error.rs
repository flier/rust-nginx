use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("out of memory")]
    OutOfMemory,

    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}
