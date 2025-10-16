// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::params`] (and re-exported [`crate::hashes`]).

#![cfg(test)]

use crate::hash::checksum::{FuzzyHashChecksum, CHECKSUM_SIZE_LONG, CHECKSUM_SIZE_NORMAL};
use crate::hashes;
use crate::internals::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};
use crate::FuzzyHashType;

#[rustfmt::skip]
#[test]
fn params_buckets() {
    assert_eq!(NUM_BUCKETS_SHORT, hashes::Short::NUMBER_OF_BUCKETS);
    assert_eq!(NUM_BUCKETS_NORMAL, hashes::Normal::NUMBER_OF_BUCKETS);
    assert_eq!(NUM_BUCKETS_NORMAL, hashes::NormalWithLongChecksum::NUMBER_OF_BUCKETS);
    assert_eq!(NUM_BUCKETS_LONG, hashes::Long::NUMBER_OF_BUCKETS);
    assert_eq!(NUM_BUCKETS_LONG, hashes::LongWithLongChecksum::NUMBER_OF_BUCKETS);
}

#[rustfmt::skip]
#[test]
fn params_checksum() {
    assert_eq!(CHECKSUM_SIZE_NORMAL, <<hashes::Short as FuzzyHashType>::ChecksumType as FuzzyHashChecksum>::SIZE);
    assert_eq!(CHECKSUM_SIZE_NORMAL, <<hashes::Normal as FuzzyHashType>::ChecksumType as FuzzyHashChecksum>::SIZE);
    assert_eq!(CHECKSUM_SIZE_NORMAL, <<hashes::Long as FuzzyHashType>::ChecksumType as FuzzyHashChecksum>::SIZE);
    assert_eq!(CHECKSUM_SIZE_LONG, <<hashes::NormalWithLongChecksum as FuzzyHashType>::ChecksumType as FuzzyHashChecksum>::SIZE);
    assert_eq!(CHECKSUM_SIZE_LONG, <<hashes::LongWithLongChecksum as FuzzyHashType>::ChecksumType as FuzzyHashChecksum>::SIZE);
}

#[test]
fn params_sizes() {
    macro_rules! test_case {
        ($ty: ty, $base: expr) => {
            assert_eq!(<$ty>::SIZE_IN_BYTES, $base);
            assert_eq!(<$ty>::LEN_IN_STR_EXCEPT_PREFIX, $base * 2);
            assert_eq!(<$ty>::LEN_IN_STR, ($base * 2) + 2);
        };
    }
    test_case!(hashes::Short, 15);
    test_case!(hashes::Normal, 35);
    test_case!(hashes::NormalWithLongChecksum, 37);
    test_case!(hashes::Long, 67);
    test_case!(hashes::LongWithLongChecksum, 69);
}
