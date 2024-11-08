//! Contains a hashing method that matches the hashing method described
//! here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)
//! This hashing method is particularly useful when you don't want to read
//! an entire file's bytes to generate a hash, provided you trust that any
//! changes to the file will cause byte differences in the first and last
//! bytes of the file, or a change to its file size.
use io::prelude::*;
use std::fs::File;
use std::{fmt, io};

#[derive(Debug)]
pub enum HashError {
    FileTooSmall,
    IoError(io::Error),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileTooSmall => write!(f, "File size too small"),
            Self::IoError(err) => write!(f, "{}", err),
        }
    }
}
impl std::error::Error for HashError {}
impl From<io::Error> for HashError {
    fn from(err: io::Error) -> Self {
        HashError::IoError(err)
    }
}

fn to_uint64(hash: &mut u64) {
    *hash &= 0xFFFFFFFFFFFFFFFF;
}

/// Hashes the file at the provided path identically to the method described
/// here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)
///
/// The file size is hashed along with the first 64KB and the last 64KB of
/// the file. This works well for media files but may not work well if only
/// interior bytes of your file are changing and the filesize remains unchanged.
///
/// The minimum file size for this method to work is 128KB, and a HashError::FileTooSmall
/// will be thrown if this is not the case. Other hashing methods that read all the bytes
/// should be employed for those files. This was a conscious decision to maintain parity
/// with the functioning of the python library.
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
    let chunk_size = 65536;
    let min_file_size = chunk_size * 2;

    let mut f = File::open(path.as_ref())?;
    let mut file_hash: u64 = f.metadata()?.len();

    if file_hash < min_file_size {
        return Err(HashError::FileTooSmall);
    }

    let mut buffer = [0; 8];
    for _ in 0..(chunk_size / 8) {
        f.read_exact(&mut buffer)?;
        file_hash = file_hash.wrapping_add(u64::from_le_bytes(buffer));
        to_uint64(&mut file_hash);
    }

    let offset: i64 = chunk_size as i64;
    f.seek(io::SeekFrom::End(-offset))?;

    for _ in 0..(chunk_size / 8) {
        f.read_exact(&mut buffer)?;
        file_hash = file_hash.wrapping_add(u64::from_le_bytes(buffer));
        to_uint64(&mut file_hash);
    }

    Ok(format!("{:016x}", file_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_hashes_properly() {
        let result = oshash("test-resources/testdata").unwrap();
        assert_eq!(result, "40d354daf3acce9c");
    }
    #[test]
    fn it_throw_error_when_input_too_small() {
        let result = oshash("test-resources/too_small");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "File size too small");
    }
    #[test]
    fn it_throw_error_if_file_does_not_exist() {
        let result = oshash("test-resources/does_not_exist");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No such file or directory"));
    }
}
