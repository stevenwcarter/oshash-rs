use std::fs::File;
use std::io::{self, Seek};
use std::path::Path;

use oshash::{oshash, oshash_buf, HashError};

use common::{TempFile, GENERATED_SIZES};

mod common;

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

#[test]
fn path_and_handle_based_hashes_agree_on_the_reference_file() {
    let path = "test-resources/testdata";
    let mut file = File::open(path).unwrap();
    let len = file.metadata().unwrap().len();

    assert_eq!(oshash(path).unwrap(), oshash_buf(&mut file, len).unwrap());
}

/// The path-based `oshash` (positioned reads) and the handle-based `oshash_buf` (seeks) take
/// different routes to the same two chunks; both must match an independent computation.
#[test]
fn path_and_handle_based_hashes_agree_on_generated_files() {
    for (tag, size) in GENERATED_SIZES {
        let temp = TempFile::generate(&format!("sync-{tag}"), size);
        let expected = temp.expected_hash();

        let mut file = File::open(temp.path()).unwrap();
        let len = file.metadata().unwrap().len();
        assert_eq!(len, size as u64, "{tag}");

        assert_eq!(oshash(temp.path()).unwrap(), expected, "{tag}: path-based");
        assert_eq!(
            oshash_buf(&mut file, len).unwrap(),
            expected,
            "{tag}: handle-based"
        );
    }
}

#[test]
fn it_reports_file_too_small_as_the_typed_variant() {
    assert!(matches!(
        oshash("test-resources/too_small"),
        Err(HashError::FileTooSmall)
    ));
}

/// One byte short of the minimum must be rejected by both entry points, not read short.
#[test]
fn it_rejects_a_file_one_byte_below_the_minimum() {
    let temp = TempFile::generate("sync-below-minimum", 2 * 65536 - 1);
    let mut file = File::open(temp.path()).unwrap();
    let len = file.metadata().unwrap().len();

    assert!(matches!(oshash(temp.path()), Err(HashError::FileTooSmall)));
    assert!(matches!(
        oshash_buf(&mut file, len),
        Err(HashError::FileTooSmall)
    ));
}
