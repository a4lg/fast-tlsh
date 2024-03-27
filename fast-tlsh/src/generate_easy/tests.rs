// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::generate_easy`].

#![cfg(test)]

use super::{hash_buf, hash_buf_for};

use crate::generate::tests::{LOREM_IPSUM, LOREM_IPSUM_HASH_NORMAL};
use crate::hashes;

#[test]
fn example_hash_buf_for_custom() {
    type CustomTlsh = hashes::Short;
    let hash = hash_buf_for::<CustomTlsh>(b"Hello, World!").unwrap();
    assert_eq!(hash.to_string(), "T1E16004017D3551777571D55C005CC5");
}

#[test]
fn example_hash_buf_for_normal() {
    type CustomTlsh = hashes::Normal;
    let hash = hash_buf_for::<CustomTlsh>(LOREM_IPSUM).unwrap();
    assert_eq!(hash.to_string(), LOREM_IPSUM_HASH_NORMAL);
}

#[test]
fn example_hash_buf() {
    let hash = hash_buf(LOREM_IPSUM).unwrap();
    assert_eq!(hash.to_string(), LOREM_IPSUM_HASH_NORMAL);
}
