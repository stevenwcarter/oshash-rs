use std::panic;

use tokio::io::{self, AsyncReadExt, AsyncSeekExt};
use tokio::task;

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
/// The whole hash runs as a single [`tokio::task::spawn_blocking`] call around the synchronous
/// [`oshash`](crate::oshash), rather than one blocking-pool hop per file operation, so it must
/// be awaited from within a Tokio runtime.
///
/// # Errors
///
/// Will return `HashError::FileTooSmall` if the file is smaller than 128kb
/// Will return any `IoError` surfaced from the filesystem, or an `IoError` if the blocking task
/// is cancelled because the runtime is shutting down
///
/// # Panics
///
/// Propagates a panic from the blocking task, should one occur.
///
/// # Example
///
/// ```
/// # tokio_test::block_on(async {
///   let result = oshash::oshash_async("test-resources/testdata").await.unwrap();
///
///   assert_eq!(result, "40d354daf3acce9c");
/// # })
/// ```
///
#[must_use = "the hash result should be used"]
pub async fn oshash_async<T: AsRef<str>>(path: T) -> Result<String, HashError> {
    let path = path.as_ref().to_owned();

    match task::spawn_blocking(move || oshash(path)).await {
        Ok(result) => result,
        Err(err) => match err.try_into_panic() {
            Ok(payload) => panic::resume_unwind(payload),
            Err(err) => Err(HashError::IoError(err.into())),
        },
    }
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
/// # tokio_test::block_on(async {
/// let mut file = tokio::fs::File::open("test-resources/testdata").await.unwrap();
/// let len = file.metadata().await.unwrap().len();
/// let result = oshash::oshash_buf_async(&mut file, len).await.unwrap();
///
/// assert_eq!(result, "40d354daf3acce9c");
/// # })
/// ```
///
#[must_use = "the hash result should be used"]
pub async fn oshash_buf_async<T>(file: &mut T, len: u64) -> Result<String, HashError>
where
    T: AsyncSeekExt + AsyncReadExt + Unpin,
{
    if len < MIN_FILE_SIZE as u64 {
        return Err(HashError::FileTooSmall);
    }

    let current_offset = file.stream_position().await?;

    let mut buffer = vec![0u8; CHUNK_SIZE];

    // Read first CHUNK_SIZE bytes
    file.seek(io::SeekFrom::Start(0)).await?;
    file.read_exact(&mut buffer).await?;
    let file_hash = add_chunk(len, &buffer);

    // Read last CHUNK_SIZE bytes
    file.seek(io::SeekFrom::End(-(CHUNK_SIZE as i64))).await?;
    file.read_exact(&mut buffer).await?;
    let file_hash = add_chunk(file_hash, &buffer);

    // Restore original position
    file.seek(io::SeekFrom::Start(current_offset)).await?;

    Ok(format_hash(file_hash))
}
