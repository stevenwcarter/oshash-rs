use std::fs::File;
use std::io::{self, Seek};

use oshash::{oshash, oshash_buf, HashError};

#[test]
fn it_hashes_properly() {
    let result = oshash("test-resources/testdata").unwrap();
    assert_eq!(result, "40d354daf3acce9c");
}

#[test]
fn it_throws_io_error() {
    let result = oshash("test-resources/dne");
    assert!(result.is_err());
    match result {
        Err(HashError::IoError(_)) => {
            // Expected error, test passes
        }
        _ => panic!("Unexpected error"),
    }
}

#[test]
fn it_throw_error_when_input_too_small() {
    let result = oshash("test-resources/too_small");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "File size too small");
}

#[test]
fn it_throws_error_if_file_does_not_exist() {
    let result = oshash("test-resources/does_not_exist");
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
