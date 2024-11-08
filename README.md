# oshash

[![codecov](https://codecov.io/github/stevenwcarter/oshash-rs/graph/badge.svg?token=JQAHCG5F3X)](https://codecov.io/github/stevenwcarter/oshash-rs)

Contains a hashing method that matches the hashing method described
here: [https://pypi.org/project/oshash/](https://pypi.org/project/oshash/)

This hashing method is particularly useful when you don’t want to read
an entire file’s bytes to generate a hash, provided you trust that any
changes to the file will cause byte differences in the first and last
bytes of the file, or a change to its file size.
