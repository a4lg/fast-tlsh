// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::hash::checksum`].

#![cfg(test)]

use super::inner::InnerChecksum;
use super::inner::OneByteChecksumChecker as OneByteChecksumCheckerTrait;
use super::{
    FuzzyHashChecksum, FuzzyHashChecksumData, OneByteChecksumChecker, CHECKSUM_SIZE_LONG,
    CHECKSUM_SIZE_NORMAL,
};

use crate::internals::buckets::{
    FuzzyHashBucketMapper, FuzzyHashBucketsInfo, NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL,
    NUM_BUCKETS_SHORT,
};
use crate::internals::errors::ParseError;

#[test]
fn one_byte_checksum_checker_48() {
    for value in u8::MIN..=u8::MAX {
        assert_eq!(
            OneByteChecksumChecker::<NUM_BUCKETS_SHORT>::is_valid(value),
            value <= NUM_BUCKETS_SHORT as u8
        );
    }
    // Test specific examples
    assert_eq!(NUM_BUCKETS_SHORT, 48);
    assert!(OneByteChecksumChecker::<NUM_BUCKETS_SHORT>::is_valid(48));
    assert!(!OneByteChecksumChecker::<NUM_BUCKETS_SHORT>::is_valid(49));
}

#[test]
fn one_byte_checksum_checker_256() {
    fn test_all_checksum_is_valid<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        OneByteChecksumChecker<SIZE_BUCKETS>: OneByteChecksumCheckerTrait,
    {
        for value in u8::MIN..=u8::MAX {
            assert!(OneByteChecksumChecker::<SIZE_BUCKETS>::is_valid(value));
        }
    }
    test_all_checksum_is_valid::<NUM_BUCKETS_NORMAL>();
    test_all_checksum_is_valid::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_impls() {
    fn test<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        let c = FuzzyHashChecksumData::<SIZE_CKSUM, SIZE_BUCKETS>::new();
        // Default data after initialization
        // (since `new()` is crate-private, Default trais is not implemented)
        assert!(c.data().iter().all(|&x| x == 0));
    }
    test::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_SHORT>();
    test::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_NORMAL>();
    test::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_LONG>();
    test::<CHECKSUM_SIZE_LONG, NUM_BUCKETS_NORMAL>();
    test::<CHECKSUM_SIZE_LONG, NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_validness_short_48() {
    for value in u8::MIN..=u8::MAX {
        let c =
            FuzzyHashChecksumData::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_SHORT>::from_raw(&[value]);
        assert_eq!(c.is_valid(), value <= NUM_BUCKETS_SHORT as u8);
    }
}

#[test]
fn checksum_validness_short_256() {
    fn test<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        for value in u8::MIN..=u8::MAX {
            let c = FuzzyHashChecksumData::<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>::from_raw(&[value]);
            assert!(c.is_valid());
        }
    }
    test::<NUM_BUCKETS_NORMAL>();
    test::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_validness_long_256() {
    fn test_all<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        for b0 in u8::MIN..=u8::MAX {
            for b1 in u8::MIN..=u8::MAX {
                for b2 in u8::MIN..=u8::MAX {
                    let value = [b0, b1, b2];
                    let c =
                        FuzzyHashChecksumData::<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>::from_raw(&value);
                    assert!(c.is_valid());
                }
            }
        }
    }
    test_all::<NUM_BUCKETS_NORMAL>();
    test_all::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_from_str_bytes_short_examples() {
    fn test<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        let mut c: Result<FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>, ParseError>;
        // Success cases (allows both lower case and upper case)
        c = FuzzyHashChecksumData::from_str_bytes(b"a1");
        assert!(c.is_ok());
        assert_eq!(c.unwrap().data()[0], 0x1a);
        c = FuzzyHashChecksumData::from_str_bytes(b"B1");
        assert!(c.is_ok());
        assert_eq!(c.unwrap().data()[0], 0x1b);
        // Failure due to invalid length
        c = FuzzyHashChecksumData::from_str_bytes(b"0");
        assert_eq!(c, Err(ParseError::InvalidStringLength));
        c = FuzzyHashChecksumData::from_str_bytes(b"000");
        assert_eq!(c, Err(ParseError::InvalidStringLength));
        // Failure due to an invalid character
        c = FuzzyHashChecksumData::from_str_bytes(b"g0");
        assert_eq!(c, Err(ParseError::InvalidCharacter));
        c = FuzzyHashChecksumData::from_str_bytes(b"0G");
        assert_eq!(c, Err(ParseError::InvalidCharacter));
    }
    test::<NUM_BUCKETS_SHORT>();
    test::<NUM_BUCKETS_NORMAL>();
    test::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_from_str_bytes_long_examples() {
    fn test<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        let mut c: Result<FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>, ParseError>;
        // Success cases (allows both lower case and upper case)
        c = FuzzyHashChecksumData::from_str_bytes(b"a1dc90");
        assert!(c.is_ok());
        assert_eq!(c.unwrap().data(), b"\x1a\xcd\x09");
        c = FuzzyHashChecksumData::from_str_bytes(b"A1DC90");
        assert!(c.is_ok());
        assert_eq!(c.unwrap().data(), b"\x1a\xcd\x09");
        // Failure due to invalid length
        c = FuzzyHashChecksumData::from_str_bytes(b"00");
        assert_eq!(c, Err(ParseError::InvalidStringLength));
        c = FuzzyHashChecksumData::from_str_bytes(b"0000");
        assert_eq!(c, Err(ParseError::InvalidStringLength));
        c = FuzzyHashChecksumData::from_str_bytes(b"0000000");
        assert_eq!(c, Err(ParseError::InvalidStringLength));
        // Failure due to an invalid character
        c = FuzzyHashChecksumData::from_str_bytes(b"g00000");
        assert_eq!(c, Err(ParseError::InvalidCharacter));
        c = FuzzyHashChecksumData::from_str_bytes(b"00000G");
        assert_eq!(c, Err(ParseError::InvalidCharacter));
    }
    test::<NUM_BUCKETS_NORMAL>();
    test::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_compare_short_binary() {
    fn test<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        for &a0 in &[0, 1] {
            let a = [a0];
            let ca = FuzzyHashChecksumData::<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>::from_raw(&a);
            for &b0 in &[0, 1] {
                let b = [b0];
                let cb = FuzzyHashChecksumData::from_raw(&b);
                let expected = (a0 ^ b0) as u32;
                assert_eq!(ca.compare(&cb), expected);
            }
        }
    }
    test::<NUM_BUCKETS_SHORT>();
    test::<NUM_BUCKETS_NORMAL>();
    test::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_compare_long_binary() {
    fn test<const SIZE_BUCKETS: usize>()
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>: FuzzyHashChecksum,
    {
        for &a0 in &[0, 1] {
            for &a1 in &[0, 1] {
                for &a2 in &[0, 1] {
                    let a = [a0, a1, a2];
                    let ca =
                        FuzzyHashChecksumData::<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>::from_raw(&a);
                    for &b0 in &[0, 1] {
                        for &b1 in &[0, 1] {
                            for &b2 in &[0, 1] {
                                let b = [b0, b1, b2];
                                let cb = FuzzyHashChecksumData::from_raw(&b);
                                let expected = ((a0 ^ b0) + (a1 ^ b1) + (a2 ^ b2)) as u32;
                                assert_eq!(ca.compare(&cb), expected);
                            }
                        }
                    }
                }
            }
        }
    }
    test::<NUM_BUCKETS_NORMAL>();
    test::<NUM_BUCKETS_LONG>();
}

#[test]
fn checksum_update_48_example() {
    // TLSH hash value: T1E16004017D3551777571D55C005CC5
    //                    ~~ checksum ("E1" == 0x1e)
    let mut state = FuzzyHashChecksumData::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_SHORT>::new();
    for window in b"Hello, World!"[3..].windows(2) {
        state.update(window[1], window[0]);
    }
    assert_eq!(state.data(), &[0x1e]);
}

#[test]
fn checksum_update_256_example() {
    // If we ignore any errors, the TLSH hash value would be:
    // T14E60440000000000000000000C00000000000000000000000000000030000000000000
    //   ~~ checksum ("4E" == 0xe4)
    let mut state = FuzzyHashChecksumData::<CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_NORMAL>::new();
    for window in b"Hello, World!"[3..].windows(2) {
        state.update(window[1], window[0]);
    }
    assert_eq!(state.data(), &[0xe4]);
}
