#[cfg(feature = "tokio")]
use std::io;
#[cfg(feature = "tokio")]
use std::path::Path;

#[cfg(feature = "tokio")]
use oshash::{oshash_async, oshash_buf_async, HashError};
#[cfg(feature = "tokio")]
use tokio::fs::File;
#[cfg(feature = "tokio")]
use tokio::io::AsyncSeekExt;

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_hashes_properly_async() {
    let path = Path::new("test-resources/testdata")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash_async(path).await.unwrap();
    assert_eq!(result, "40d354daf3acce9c");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_throws_io_error_async() {
    let path = Path::new("test-resources/dne")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash_async(path).await;
    assert!(result.is_err());
    match result {
        Err(HashError::IoError(_)) => {
            let _ = result.unwrap_err().to_string();
            // Expected error, test passes
        }
        _ => panic!("Unexpected error"),
    }
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_throw_error_when_input_too_small() {
    let path = Path::new("test-resources/too_small")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash_async(path).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "File size too small");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_throws_error_if_file_does_not_exist() {
    let path = Path::new("test-resources/does_not_exist")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash_async(path).await;
    assert!(result.is_err());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_accepts_seek_and_confirms_seeks_and_leave_seek_at_original_offset() {
    let mut file = File::open("test-resources/testdata").await.unwrap();
    let len = file.metadata().await.unwrap().len();
    let offset = 10;
    file.seek(io::SeekFrom::Start(offset)).await.unwrap();
    let result = oshash_buf_async(&mut file, len).await.unwrap();
    assert_eq!(result, "40d354daf3acce9c");

    assert_eq!(file.stream_position().await.unwrap(), offset);
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn it_throws_error_when_input_too_small_for_buf() {
    let mut file = File::open("test-resources/too_small").await.unwrap();
    let len = file.metadata().await.unwrap().len();
    let result = oshash_buf_async(&mut file, len).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "File size too small");
}
