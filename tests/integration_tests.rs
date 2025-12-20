use std::fs::File;
use std::io::{self, Seek};
use std::path::Path;

use oshash::{oshash, oshash_buf, HashError};

#[test]
fn it_hashes_properly() {
    let path = Path::new("test-resources/testdata")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash(path).unwrap();
    assert_eq!(result, "40d354daf3acce9c");
}

#[test]
fn it_throws_io_error() {
    let path = Path::new("test-resources/dne")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash(path);
    assert!(result.is_err());
    match result {
        Err(HashError::IoError(_)) => {
            let _ = result.unwrap_err().to_string();
            // Expected error, test passes
        }
        _ => panic!("Unexpected error"),
    }
}

#[test]
fn it_throw_error_when_input_too_small() {
    let path = Path::new("test-resources/too_small")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash(path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "File size too small");
}

#[test]
fn it_throws_error_if_file_does_not_exist() {
    let path = Path::new("test-resources/does_not_exist")
        .as_os_str()
        .to_str()
        .unwrap();
    let result = oshash(path);
    assert!(result.is_err());
}

#[test]
fn it_accepts_seek_and_confirms_seeks_and_leave_seek_at_original_offset() {
    let mut file = File::open("test-resources/testdata").unwrap();
    let len = file.metadata().unwrap().len();
    let offset = 10;
    file.seek(io::SeekFrom::Start(offset)).unwrap();
    let result = oshash_buf(&mut file, len).unwrap();
    assert_eq!(result, "40d354daf3acce9c");

    assert_eq!(file.stream_position().unwrap(), offset);
}

#[test]
fn it_throws_error_when_input_too_small_for_buf() {
    let mut file = File::open("test-resources/too_small").unwrap();
    let len = file.metadata().unwrap().len();
    let result = oshash_buf(&mut file, len);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "File size too small");
}
