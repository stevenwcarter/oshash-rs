use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncSeekExt};

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
/// # tokio_test::block_on(async {
///   let result = oshash::oshash_async("test-resources/testdata").await.unwrap();
///
///   assert_eq!(result, "40d354daf3acce9c");
/// # })
/// ```
///
#[must_use = "the hash result should be used"]
pub async fn oshash_async<T: AsRef<str>>(path: T) -> Result<String, HashError> {
    let mut f = File::open(path.as_ref()).await?;
    let len: u64 = f.metadata().await?.len();

    oshash_buf_async(&mut f, len).await
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
    let result = oshash_buf_async_inner(file, len).await;
    // Always restore the original seek position, even on error
    let _ = file.seek(io::SeekFrom::Start(current_offset)).await;
    result
}

async fn oshash_buf_async_inner<T>(file: &mut T, len: u64) -> Result<String, HashError>
where
    T: AsyncSeekExt + AsyncReadExt + Unpin,
{
    let mut file_hash: u64 = len;
    let mut buffer = vec![0u8; CHUNK_SIZE];

    file.seek(io::SeekFrom::Start(0)).await?;
    file.read_exact(&mut buffer).await?;
    accumulate(&mut file_hash, &buffer);

    file.seek(io::SeekFrom::End(-(CHUNK_SIZE as i64))).await?;
    file.read_exact(&mut buffer).await?;
    accumulate(&mut file_hash, &buffer);

    Ok(format!("{file_hash:016x}"))
}
