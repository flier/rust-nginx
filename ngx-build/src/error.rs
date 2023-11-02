use std::result::Result as StdResult;

use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("missing argument: {0}")]
    MissingArgument(&'static str),

    #[error("execute command failed")]
    ExecuteError,

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "gen")]
    #[error(transparent)]
    BindgenError(#[from] bindgen::BindgenError),
}
