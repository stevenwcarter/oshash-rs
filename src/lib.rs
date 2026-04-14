#![warn(clippy::all)]
//! Contains a hashing method that matches the hashing method described
//! here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)
//! This hashing method is particularly useful when you don't want to read
//! an entire file's bytes to generate a hash, provided you trust that any
//! changes to the file will cause byte differences in the first and last
//! bytes of the file, or a change to its file size.
use std::{fmt, io};

#[cfg(feature = "tokio")]
mod async_impl;

#[cfg(feature = "tokio")]
pub use async_impl::{oshash_async, oshash_buf_async};
pub use sync::{oshash, oshash_buf};

mod sync;

const CHUNK_SIZE: usize = 65536;
const MIN_FILE_SIZE: usize = 2 * CHUNK_SIZE;

#[derive(Debug)]
pub enum HashError {
    FileTooSmall,
    IoError(io::Error),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileTooSmall => write!(f, "File size too small"),
            Self::IoError(err) => write!(f, "{err}"),
        }
    }
}
impl std::error::Error for HashError {}
impl From<io::Error> for HashError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
