// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::compare`].

#![cfg(test)]

use core::str::FromStr;

use crate::{FuzzyHashType, Tlsh};

#[test]
fn tlsh_timing_unittest_vectors() {
    // Displayed in the official implementation's timing_unittest.
    let hash1 = "T1A12500088C838B0A0F0EC3C0ACAB82F3B8228B0308CFA302338C0F0AE2C24F28000008";
    let hash2 = "T129251210F4C18D0A5F0661C4F64D905B585253A3024F022323E5074CC5601904886D1C";
    let hash1 = Tlsh::from_str(hash1).unwrap();
    let hash2 = Tlsh::from_str(hash2).unwrap();
    let expected = 138;
    assert_eq!(hash1.compare(&hash2), expected);
    assert_eq!(hash2.compare(&hash1), expected);
}
