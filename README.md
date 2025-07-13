# oshash

[![Crates.io](https://img.shields.io/crates/v/oshash.svg)](https://crates.io/crates/oshash)
[![Documentation](https://docs.rs/oshash/badge.svg)](https://docs.rs/oshash/)
[![Codecov](https://codecov.io/github/stevenwcarter/oshash-rs/coverage.svg?branch=main)](https://codecov.io/gh/stevenwcarter/oshash-rs)
[![Dependency status](https://deps.rs/repo/github/stevenwcarter/oshash-rs/status.svg)](https://deps.rs/repo/github/stevenwcarter/oshash-rs)

Contains a hashing method that matches the hashing method described
here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)

This hashing method is particularly useful when you don’t want to read
an entire file’s bytes to generate a hash, provided you trust that any
changes to the file will cause byte differences in the first and last
bytes of the file, or a change to its file size.

### CLI Utility

A command line utility is provided to generate hashes for files specified as arguments.

```
$ oshash test-resources/testdata
40d354daf3acce9c test-resources/testdata
```
