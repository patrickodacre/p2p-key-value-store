pub use crate::error::KvsError;

pub type Result<T> = core::result::Result<T, KvsError>;

pub struct W<T>(pub T);
