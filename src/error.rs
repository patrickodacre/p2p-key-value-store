use std::backtrace::Backtrace;
use std::io::Error as IoError;

#[derive(thiserror::Error, Debug)]
pub enum KvsError {
    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Entry not found: {0}")]
    NotFound(String),
}
