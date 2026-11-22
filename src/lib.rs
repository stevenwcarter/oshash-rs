#![warn(clippy::all)]
#![deny(missing_docs)]
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

/// Error type returned by oshash functions.
#[derive(Debug)]
#[non_exhaustive]
pub enum HashError {
    /// The file is smaller than the minimum required size (128 KB).
    FileTooSmall,
    /// An I/O error occurred while reading the file.
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

/// Adds every little-endian `u64` in `chunk` to `hash`, wrapping on overflow.
///
/// `chunk` is always a `CHUNK_SIZE` buffer, so its length is a multiple of 8 and no trailing
/// bytes are dropped.
fn add_chunk(hash: u64, chunk: &[u8]) -> u64 {
    chunk.chunks_exact(8).fold(hash, |hash, word| {
        hash.wrapping_add(u64::from_le_bytes(
            word.try_into().expect("chunk size is 8"),
        ))
    })
}

/// Renders the final hash the way the Python library does: 16 lowercase hex digits.
fn format_hash(hash: u64) -> String {
    format!("{hash:016x}")
}
