use io::prelude::*;
use std::fs::File;
use std::io;
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

use super::*;

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
#[must_use = "the hash result should be used"]
pub fn oshash<T: AsRef<str>>(path: T) -> Result<String, HashError> {
    let f = File::open(path.as_ref())?;
    let len: u64 = f.metadata()?.len();

    hash_fresh_file(f, len)
}

/// Hashes a freshly opened file with positioned reads, which need no seeks: there is no
/// caller-visible offset to preserve on a handle nobody else has seen.
#[cfg(any(unix, windows))]
fn hash_fresh_file(file: File, len: u64) -> Result<String, HashError> {
    if len < MIN_FILE_SIZE as u64 {
        return Err(HashError::FileTooSmall);
    }

    let mut buffer = [0u8; CHUNK_SIZE];

    read_exact_at(&file, &mut buffer, 0)?;
    let file_hash = add_chunk(len, &buffer);

    read_exact_at(&file, &mut buffer, len - CHUNK_SIZE as u64)?;
    let file_hash = add_chunk(file_hash, &buffer);

    Ok(format_hash(file_hash))
}

/// Targets without a positioned-read API fall back to the seek-based implementation.
#[cfg(not(any(unix, windows)))]
fn hash_fresh_file(mut file: File, len: u64) -> Result<String, HashError> {
    oshash_buf(&mut file, len)
}

#[cfg(unix)]
fn read_exact_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    file.read_exact_at(buf, offset)
}

/// Windows has no `read_exact` counterpart of `seek_read`, so loop over short reads the same way
/// `Read::read_exact` does.
#[cfg(windows)]
fn read_exact_at(file: &File, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
    while !buf.is_empty() {
        match file.seek_read(buf, offset) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                ));
            }
            Ok(read) => {
                let rest = buf;
                buf = &mut rest[read..];
                offset += read as u64;
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }

    Ok(())
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
#[must_use = "the hash result should be used"]
pub fn oshash_buf<T>(file: &mut T, len: u64) -> Result<String, HashError>
where
    T: Seek + Read,
{
    if len < MIN_FILE_SIZE as u64 {
        return Err(HashError::FileTooSmall);
    }

    let current_offset = file.stream_position()?;

    let mut buffer = [0u8; CHUNK_SIZE];

    // Read first CHUNK_SIZE bytes
    file.seek(io::SeekFrom::Start(0))?;
    file.read_exact(&mut buffer)?;
    let file_hash = add_chunk(len, &buffer);

    // Read last CHUNK_SIZE bytes
    file.seek(io::SeekFrom::End(-(CHUNK_SIZE as i64)))?;
    file.read_exact(&mut buffer)?;
    let file_hash = add_chunk(file_hash, &buffer);

    // Restore original position
    file.seek(io::SeekFrom::Start(current_offset))?;

    Ok(format_hash(file_hash))
}
