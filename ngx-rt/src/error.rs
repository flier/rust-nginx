use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("out of memory")]
    OutOfMemory,

    #[error("internal error, {0}")]
    InternalError(isize),

    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}

impl From<isize> for Error {
    fn from(value: isize) -> Self {
        Self::InternalError(value)
    }
}
