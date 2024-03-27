// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: `crate::generate::bucket_aggregation::portable_simd`.

#![cfg(test)]

use super::INTERLEAVE_AS_DIBITS_TABLE;

#[test]
fn interleave_as_dibits_table_values() {
    let table = INTERLEAVE_AS_DIBITS_TABLE;
    const ZERO_INDEX: usize = 0;
    const ZERO_VALUE: u8 = 0;
    assert_eq!(&table[0b10000000][ZERO_INDEX], &[0b10000000, ZERO_VALUE]);
    assert_eq!(&table[0b01000000][ZERO_INDEX], &[0b00100000, ZERO_VALUE]);
    assert_eq!(&table[0b00100000][ZERO_INDEX], &[0b00001000, ZERO_VALUE]);
    assert_eq!(&table[0b00010000][ZERO_INDEX], &[0b00000010, ZERO_VALUE]);
    assert_eq!(&table[0b00001000][ZERO_INDEX], &[ZERO_VALUE, 0b10000000]);
    assert_eq!(&table[0b00000100][ZERO_INDEX], &[ZERO_VALUE, 0b00100000]);
    assert_eq!(&table[0b00000010][ZERO_INDEX], &[ZERO_VALUE, 0b00001000]);
    assert_eq!(&table[0b00000001][ZERO_INDEX], &[ZERO_VALUE, 0b00000010]);
    assert_eq!(&table[ZERO_INDEX][0b10000000], &[0b01000000, ZERO_VALUE]);
    assert_eq!(&table[ZERO_INDEX][0b01000000], &[0b00010000, ZERO_VALUE]);
    assert_eq!(&table[ZERO_INDEX][0b00100000], &[0b00000100, ZERO_VALUE]);
    assert_eq!(&table[ZERO_INDEX][0b00010000], &[0b00000001, ZERO_VALUE]);
    assert_eq!(&table[ZERO_INDEX][0b00001000], &[ZERO_VALUE, 0b01000000]);
    assert_eq!(&table[ZERO_INDEX][0b00000100], &[ZERO_VALUE, 0b00010000]);
    assert_eq!(&table[ZERO_INDEX][0b00000010], &[ZERO_VALUE, 0b00000100]);
    assert_eq!(&table[ZERO_INDEX][0b00000001], &[ZERO_VALUE, 0b00000001]);
}
