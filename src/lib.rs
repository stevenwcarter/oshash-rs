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

pub fn oshash(path: &str) -> Result<String, HashError> {
    let chunk_size = 65536;
    let min_file_size = chunk_size * 2;

    let mut f = File::open(path)?;
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
}
