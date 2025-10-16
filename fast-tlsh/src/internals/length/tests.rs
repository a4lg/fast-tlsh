// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::length`].

#![cfg(test)]

use super::{
    ConstrainedLengthProcessingInfo, DataLengthProcessingMode, DataLengthValidity,
    ENCODED_INDICES_BY_LEADING_ZEROS, ENCODED_VALUE_SIZE, FuzzyHashLengthEncoding,
    LengthProcessingInfo, TOP_VALUE_BY_ENCODING, encode, naive,
};

use crate::internals::buckets::{
    FuzzyHashBucketMapper, FuzzyHashBucketsInfo, NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL,
    NUM_BUCKETS_SHORT,
};
use crate::internals::errors::ParseError;

#[test]
fn len_prerequisites() {
    // Maximum size for the encoding 0 must be 1 for current algorithm of "encode".
    assert_eq!(TOP_VALUE_BY_ENCODING[0], 1);
    for len_range in TOP_VALUE_BY_ENCODING.as_slice().windows(2) {
        let bottom = len_range[0];
        let top = len_range[1];
        // TOP_VALUE_BY_ENCODING must be a strictly increasing array.
        assert!(bottom < top);
        // Count of leading zeros must be decreasing but must not too steep.
        let clz_b = bottom.leading_zeros();
        let clz_t = top.leading_zeros();
        let clz_diff = clz_b.checked_sub(clz_t).unwrap();
        assert!(clz_diff <= 1);
    }
}

#[test]
fn length_processing_info_params() {
    fn test_params<T: ConstrainedLengthProcessingInfo>() {
        // Implementation-defined limit: MIN != 0
        assert_ne!(T::MIN, 0);
        // Implementation-defined limit: MAX != u32::MAX
        assert_ne!(T::MAX, u32::MAX);
        // Hard constraint: MAX == module's MAX
        assert_eq!(T::MAX, super::MAX);
        // Hard constraints: MIN <= MIN_CONSERVATIVE <= MAX
        assert!(T::MIN <= T::MIN_CONSERVATIVE && T::MIN_CONSERVATIVE <= T::MAX);
        // Implementation-defined limit: MIN_CONSERVATIVE < MAX
        assert!(T::MIN_CONSERVATIVE < T::MAX);
    }
    test_params::<LengthProcessingInfo<NUM_BUCKETS_SHORT>>();
    test_params::<LengthProcessingInfo<NUM_BUCKETS_NORMAL>>();
    test_params::<LengthProcessingInfo<NUM_BUCKETS_LONG>>();
}

#[test]
fn data_length_processing_mode_default() {
    // Check its default value.
    assert_eq!(
        <DataLengthProcessingMode as Default>::default(),
        DataLengthProcessingMode::Optimistic
    );
}

#[test]
fn data_length_validity_values_and_lengths() {
    // Both minimum and maximum sizes are valid.
    fn test_lengths<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        // Size 0 is too small for all modes.
        assert_eq!(
            DataLengthValidity::new::<SIZE_BUCKETS>(0),
            DataLengthValidity::TooSmall
        );
        // MIN - 1
        assert_eq!(
            DataLengthValidity::new::<SIZE_BUCKETS>(LengthProcessingInfo::<SIZE_BUCKETS>::MIN - 1),
            DataLengthValidity::TooSmall
        );
        // MIN
        if LengthProcessingInfo::<SIZE_BUCKETS>::MIN
            < LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE
        {
            // MIN < MIN_CONSERVATIVE
            assert_eq!(
                DataLengthValidity::new::<SIZE_BUCKETS>(LengthProcessingInfo::<SIZE_BUCKETS>::MIN),
                DataLengthValidity::ValidWhenOptimistic
            );
            assert_eq!(
                DataLengthValidity::new::<SIZE_BUCKETS>(
                    LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE - 1
                ),
                DataLengthValidity::ValidWhenOptimistic
            );
        } else {
            // MIN == MIN_CONSERVATIVE
            assert_eq!(
                DataLengthValidity::new::<SIZE_BUCKETS>(LengthProcessingInfo::<SIZE_BUCKETS>::MIN),
                DataLengthValidity::Valid
            );
        }
        // MIN_CONSERVATIVE
        assert_eq!(
            DataLengthValidity::new::<SIZE_BUCKETS>(
                LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE
            ),
            DataLengthValidity::Valid
        );
        // MAX
        assert_eq!(
            DataLengthValidity::new::<SIZE_BUCKETS>(LengthProcessingInfo::<SIZE_BUCKETS>::MAX),
            DataLengthValidity::Valid
        );
        // MAX + 1
        assert_eq!(
            DataLengthValidity::new::<SIZE_BUCKETS>(LengthProcessingInfo::<SIZE_BUCKETS>::MAX + 1),
            DataLengthValidity::TooLarge
        );
    }
    test_lengths::<NUM_BUCKETS_SHORT>();
    test_lengths::<NUM_BUCKETS_NORMAL>();
    test_lengths::<NUM_BUCKETS_LONG>();
}

#[test]
fn data_length_validity_values_and_errors() {
    assert!(DataLengthValidity::TooSmall.is_err());
    assert!(DataLengthValidity::TooLarge.is_err());
    assert!(!DataLengthValidity::Valid.is_err());
    assert!(!DataLengthValidity::ValidWhenOptimistic.is_err());
    for &mode in &[
        DataLengthProcessingMode::Conservative,
        DataLengthProcessingMode::Optimistic,
    ] {
        assert!(DataLengthValidity::TooSmall.is_err_on(mode));
        assert!(DataLengthValidity::TooLarge.is_err_on(mode));
        assert!(!DataLengthValidity::Valid.is_err_on(mode));
    }
    assert!(
        DataLengthValidity::ValidWhenOptimistic.is_err_on(DataLengthProcessingMode::Conservative)
    );
    assert!(
        !DataLengthValidity::ValidWhenOptimistic.is_err_on(DataLengthProcessingMode::Optimistic)
    );
}

#[test]
fn length_encoding_raw() {
    for lvalue in u8::MIN..=u8::MAX {
        assert_eq!(FuzzyHashLengthEncoding::from_raw(lvalue).value(), lvalue);
    }
}

#[test]
fn length_encoding_str_examples() {
    // Invalid lengths
    for &bytes in &[b"" as &[u8], b"0", b"000"] {
        assert_eq!(
            FuzzyHashLengthEncoding::from_str_bytes(bytes),
            Err(ParseError::InvalidStringLength)
        );
    }
    assert_eq!(
        FuzzyHashLengthEncoding::from_str_bytes(b"00"),
        Ok(FuzzyHashLengthEncoding::from_raw(0x00))
    );
    assert_eq!(
        FuzzyHashLengthEncoding::from_str_bytes(b"12"),
        Ok(FuzzyHashLengthEncoding::from_raw(0x21))
    );
}

#[test]
fn length_encoding_from_str_bytes_endianness() {
    for value in u8::MIN..=u8::MAX {
        let s: String = format!("{value:02X}").chars().rev().collect();
        assert_eq!(
            FuzzyHashLengthEncoding::from_str_bytes(s.as_bytes()),
            Ok(FuzzyHashLengthEncoding::from_raw(value))
        );
    }
}

#[test]
fn length_encoding_validity() {
    // Validness corresponds to ENCODED_VALUE_SIZE.
    for lvalue in u8::MIN..=u8::MAX {
        let l = FuzzyHashLengthEncoding::from_raw(lvalue);
        assert_eq!(l.is_valid(), (l.value() as usize) < ENCODED_VALUE_SIZE);
    }
    // Both minimum and maximum sizes are valid.
    fn test_border_sizes<T: ConstrainedLengthProcessingInfo>() {
        assert!(FuzzyHashLengthEncoding::try_from(T::MIN).is_ok());
        assert!(FuzzyHashLengthEncoding::try_from(T::MIN_CONSERVATIVE).is_ok());
        assert!(FuzzyHashLengthEncoding::try_from(T::MAX).is_ok());
    }
    test_border_sizes::<LengthProcessingInfo<NUM_BUCKETS_SHORT>>();
    test_border_sizes::<LengthProcessingInfo<NUM_BUCKETS_NORMAL>>();
    test_border_sizes::<LengthProcessingInfo<NUM_BUCKETS_LONG>>();
}

#[test]
fn length_encoding_gap_between_ranges() {
    let values: Vec<_> = (u8::MIN..=u8::MAX).collect();
    for window in values.windows(2) {
        let bottom = window[0];
        let top = window[1];
        let range1 = FuzzyHashLengthEncoding::from_raw(bottom).range();
        let range2 = FuzzyHashLengthEncoding::from_raw(top).range();
        if let (Some(range1), Some(range2)) = (range1, range2) {
            // End of range 1 does not overlap with the start of range 2
            // but there's no values between range 1 and 2.
            assert_eq!(*range1.end() + 1, *range2.start());
        }
    }
}

#[test]
fn test_encode() {
    // Test for:
    // 1. Small values
    // 2. Around 2^n
    // 3. Around each top value
    // 4. Around maximum value of u32
    for len in (0..300u32)
        .chain(
            (0..u32::BITS)
                .filter(|&x| 1u32 << x >= 5)
                .flat_map(|x| ((1u32 << x) - 5)..((1u32 << x) + 5)),
        )
        .chain(
            TOP_VALUE_BY_ENCODING
                .as_slice()
                .iter()
                .filter(|&&x| x >= 5)
                .flat_map(|&x| (x - 5)..(x + 5)),
        )
        .chain((u32::MAX - 5)..=u32::MAX)
    {
        assert_eq!(super::encode(len), naive::encode(len));
    }
}

#[test]
fn encode_top_and_above() {
    for (i, &top) in TOP_VALUE_BY_ENCODING.as_slice().iter().enumerate() {
        let encoding = i as u8;
        assert_eq!(encode(top), Some(encoding));
        assert_ne!(top, u32::MAX);
        let above_top = encode(top + 1);
        assert!(above_top.is_none() || above_top == Some(encoding + 1));
    }
}

#[test]
fn property_encoded_indices_by_leading_zeros() {
    for (i, index_range) in ENCODED_INDICES_BY_LEADING_ZEROS
        .as_slice()
        .windows(2)
        .enumerate()
    {
        let clz = i as u32;
        let bottom = index_range[1];
        let top = index_range[0];
        // TOP_VALUE_BY_ENCODING denoted by each ENCODED_INDICES_BY_LEADING_ZEROS
        // shall not be empty.
        assert!(!TOP_VALUE_BY_ENCODING[bottom..top].is_empty());
        // All top values in range shall have the same number of leading zeros.
        for &x in &TOP_VALUE_BY_ENCODING[bottom..top] {
            assert_eq!(x.leading_zeros(), clz);
        }
    }
}
