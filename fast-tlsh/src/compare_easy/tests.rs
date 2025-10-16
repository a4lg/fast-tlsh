// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::compare_easy`].

#![cfg(test)]

use super::{compare, compare_with};

use crate::hashes;
use crate::internals::errors::{ParseError, ParseErrorSide};

#[test]
fn test_compare_with() {
    // Comparison succeeds.
    let result = compare_with::<hashes::Short>(
        "T140D5F17F44F8AB007AE2AC46E515DC",
        "T140D5F17F44FCAB007AE2A846E515DC",
    );
    assert_eq!(result, Ok(2));

    const HASH_OK: &str = "T140D5F17F44F8AB007AE2AC46E515DC";
    const HASH_ERR: &str = "TNULL";
    // Left side fails.
    let result = compare_with::<hashes::Short>(HASH_ERR, HASH_OK);
    let err = result.unwrap_err();
    assert_eq!(err.side(), ParseErrorSide::Left);
    assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
    // Right side fails.
    let result = compare_with::<hashes::Short>(HASH_OK, HASH_ERR);
    let err = result.unwrap_err();
    assert_eq!(err.side(), ParseErrorSide::Right);
    assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
}

#[test]
fn test_compare() {
    // Comparison succeeds.
    let result = compare(
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488",
        "T1EDD5BE96FFE41D1BCC268C7699AE4720B7B2A0032716DBAF1848A7647DD77C0DF16488",
    );
    assert_eq!(result, Ok(9));

    const HASH_OK: &str =
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488";
    const HASH_ERR: &str = "TNULL";
    // Left side fails.
    let result = compare(HASH_ERR, HASH_OK);
    let err = result.unwrap_err();
    assert_eq!(err.side(), ParseErrorSide::Left);
    assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
    // Right side fails.
    let result = compare(HASH_OK, HASH_ERR);
    let err = result.unwrap_err();
    assert_eq!(err.side(), ParseErrorSide::Right);
    assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
}
