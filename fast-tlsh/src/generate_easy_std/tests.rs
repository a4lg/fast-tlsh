// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::generate_easy_std`].

#![cfg(test)]

use super::{hash_file, hash_file_for, hash_stream, hash_stream_for};

use std::fs::File;
use std::io::Read;

use crate::errors::{GeneratorError, GeneratorOrIOError};
use crate::hashes;

const NONEXISTENT_PATH: &str = "data/examples/nonexistent_path";
const EMPTY_PATH: &str = "data/examples/empty.bin";

const SMALL_EXE_PATH: &str = "data/examples/smallexe.exe";
const SMALL_EXE_TLSH_SHORT: &str = "T140E0483A5DFC1B073D86A4A2C55A43";
const SMALL_EXE_TLSH_NORMAL: &str =
    "T1FFE04C037F895471D42E5530499E47473757E5E456D28B13ED1944654C8534C7CE9E01";

#[test]
fn example_hash_stream_for_custom_file() {
    type CustomTlsh = hashes::Short;
    let mut stream = File::open(SMALL_EXE_PATH).unwrap();
    let fuzzy_hash: CustomTlsh = hash_stream_for(&mut stream).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_SHORT);
}

#[test]
fn example_hash_stream_for_err_stream() {
    type CustomTlsh = hashes::Short;
    // Custom Read implementation (which always fails)
    struct IOFail;
    impl Read for IOFail {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::Other))
        }
    }
    let result = hash_stream_for::<CustomTlsh, _>(&mut IOFail);
    println!("{result:?}");
}

#[test]
fn example_hash_stream_for_normal_file() {
    type CustomTlsh = hashes::Normal;
    let mut stream = File::open(SMALL_EXE_PATH).unwrap();
    let fuzzy_hash: CustomTlsh = hash_stream_for(&mut stream).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_NORMAL);
}

#[test]
fn example_hash_stream_file() {
    let mut stream = File::open(SMALL_EXE_PATH).unwrap();
    let fuzzy_hash = hash_stream(&mut stream).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_NORMAL);
}

#[test]
fn example_hash_file_for_custom() {
    type CustomTlsh = hashes::Short;
    let fuzzy_hash: CustomTlsh = hash_file_for(SMALL_EXE_PATH).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_SHORT);
}

#[test]
fn example_hash_file_for_normal() {
    type CustomTlsh = hashes::Normal;
    let fuzzy_hash: CustomTlsh = hash_file_for(SMALL_EXE_PATH).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_NORMAL);
}

#[test]
fn example_hash_file() {
    let fuzzy_hash = hash_file(SMALL_EXE_PATH).unwrap();
    assert_eq!(fuzzy_hash.to_string(), SMALL_EXE_TLSH_NORMAL);
}

#[test]
fn example_hash_file_nonexistent() {
    let result = hash_file(NONEXISTENT_PATH);
    assert!(matches!(
        result,
        Err(GeneratorOrIOError::IOError(err)) if err.kind() == std::io::ErrorKind::NotFound
    ));
}

#[test]
fn example_hash_file_empty() {
    let result = hash_file(EMPTY_PATH);
    assert!(matches!(
        result,
        Err(GeneratorOrIOError::GeneratorError(
            GeneratorError::TooSmallInput
        ))
    ));
}
