//! End-to-end checks of the `oshash` binary's output format.
use std::process::{Command, Output};

const TEST_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test-resources/testdata");
const TOO_SMALL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test-resources/too_small");
const HASH: &str = "40d354daf3acce9c";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oshash"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

#[test]
fn it_prints_one_hash_and_path_line_per_file_in_order() {
    let output = run(&[TEST_FILE, TEST_FILE]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        format!("{HASH} {TEST_FILE}\n{HASH} {TEST_FILE}\n")
    );
}

#[test]
fn bench_mode_prints_the_hashes_once_then_the_summary() {
    let output = run(&["--bench", TEST_FILE]);
    let text = stdout(&output);
    let mut lines = text.lines();

    assert!(output.status.success());
    assert_eq!(lines.next(), Some(format!("{HASH} {TEST_FILE}").as_str()));
    let summary = lines.next().unwrap();
    assert!(
        summary.starts_with("Processed 1 files 1000x in "),
        "{summary}"
    );
    assert_eq!(lines.next(), None);
}

#[test]
fn it_fails_without_files() {
    let output = run(&[]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
}

/// Lines for files hashed before a failure must still reach stdout.
#[test]
fn it_keeps_earlier_output_when_a_later_file_fails() {
    let output = run(&[TEST_FILE, TOO_SMALL]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), format!("{HASH} {TEST_FILE}\n"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to hash file"));
}
