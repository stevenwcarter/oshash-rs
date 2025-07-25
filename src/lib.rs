#![warn(clippy::all)]
//! Contains a hashing method that matches the hashing method described
//! here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)
//! This hashing method is particularly useful when you don't want to read
//! an entire file's bytes to generate a hash, provided you trust that any
//! changes to the file will cause byte differences in the first and last
//! bytes of the file, or a change to its file size.
use io::prelude::*;
use std::fs::File;
use std::{fmt, io};

const CHUNK_SIZE: u64 = 65536;

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

fn to_uint64(hash: &mut u64) {
    *hash &= 0xFFFF_FFFF_FFFF_FFFF;
}

/// Hashes the file at the provided path identically to the method described
/// here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)
///
/// The file size is hashed along with the first 64KB and the last 64KB of
/// the file. This works well for media files but may not work well if only
/// interior bytes of your file are changing and the filesize remains unchanged.
///
/// The minimum file size for this method to work is 128KB, and a
/// `HashError::FileTooSmall` will be thrown if this is not the case. Other
/// hashing methods that read all the bytes should be employed for those
/// files. This was a conscious decision to maintain parity with the
/// functioning of the python library.
///
/// # Errors
///
/// Will return `HashError::FileTooSmall` if the file is smaller than 128kb
/// Will return any `IoError` surfaced from the filesystem
///
/// # Example
///
/// ```
/// let result = oshash::oshash("test-resources/testdata").unwrap();
///
/// assert_eq!(result, "40d354daf3acce9c");
/// ```
///
pub fn oshash<T: AsRef<str>>(path: T) -> Result<String, HashError> {
    let mut f = File::open(path.as_ref())?;
    let len: u64 = f.metadata()?.len();

    oshash_buf(&mut f, len)
}

/// Hashes a `Read + Seek` input if you already have a file handle. If the
/// file has an existing seek offset, then it will be reset back to that
/// position when the function exits
///
/// # Errors
///
/// Will return `HashError::FileTooSmall` if the file is smaller than 128kb
/// Will return any `IoError` surfaced from the filesystem
///
/// # Example
///
/// ```
/// let mut file = std::fs::File::open("test-resources/testdata").unwrap();
/// let len = file.metadata().unwrap().len();
/// let result = oshash::oshash_buf(&mut file, len).unwrap();
///
/// assert_eq!(result, "40d354daf3acce9c");
/// ```
///
pub fn oshash_buf<T>(file: &mut T, len: u64) -> Result<String, HashError>
where
    T: Seek + Read,
{
    const MIN_FILE_SIZE: usize = (CHUNK_SIZE * 2) as usize;
    if len < MIN_FILE_SIZE as u64 {
        return Err(HashError::FileTooSmall);
    }

    let current_offset = file.stream_position()?;

    let mut file_hash: u64 = len;

    let mut buffer = vec![0u8; CHUNK_SIZE as usize];

    // Read first CHUNK_SIZE bytes
    file.seek(io::SeekFrom::Start(0))?;
    file.read_exact(&mut buffer)?;

    for chunk in buffer.chunks_exact(8) {
        file_hash = file_hash.wrapping_add(u64::from_le_bytes(chunk.try_into().unwrap()));
        to_uint64(&mut file_hash);
    }

    // Read last CHUNK_SIZE bytes
    file.seek(io::SeekFrom::End(-(CHUNK_SIZE as i64)))?;
    file.read_exact(&mut buffer)?;

    for chunk in buffer.chunks_exact(8) {
        file_hash = file_hash.wrapping_add(u64::from_le_bytes(chunk.try_into().unwrap()));
        to_uint64(&mut file_hash);
    }

    // Restore original position
    file.seek(io::SeekFrom::Start(current_offset))?;

    Ok(format!("{file_hash:016x}"))
}
