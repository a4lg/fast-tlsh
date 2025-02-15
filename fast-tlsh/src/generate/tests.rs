// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::generate`].

#![cfg(test)]

use super::{ConstrainedFuzzyHashType, GeneratorOptions, GeneratorType, WINDOW_SIZE};

use core::fmt::Debug;
use core::str::FromStr;

use crate::buckets::constrained::{FuzzyHashBucketMapper, FuzzyHashBucketsInfo};
use crate::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};
use crate::errors::{GeneratorError, GeneratorErrorCategory};
use crate::hashes;
use crate::length::{
    ConstrainedLengthProcessingInfo, DataLengthProcessingMode, LengthProcessingInfo,
};
use crate::{Tlsh, TlshGenerator, TlshGeneratorFor};

pub(crate) const LOREM_IPSUM: &[u8] = b"Lorem ipsum dolor sit amet, consectetur \
adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna \
aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi \
ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint \
occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim \
id est laborum.";
pub(crate) const LOREM_IPSUM_HASH_NORMAL: &str =
    "T1DCF0DC36520C1B007FD32079B226559FD998A0200725E75AFCEAC99F5881184A4B1AA2";

// Data that will fill specific number of buckets.
// They are normally statistically unbalanced or on the border of "unbalance"
// in the TLSH metrics.
const BUCKETS_FILLED_12_OF_48: &[u8; 10] = b"\x73\x28\x65\xba\xeb\x85\x57\x96\x0c\xea";
const BUCKETS_FILLED_13_OF_48: &[u8; 10] = b"\x41\x3d\xad\xa3\x16\x7f\x2d\xde\xad\xec";
const BUCKETS_FILLED_17_OF_48: &[u8; 10] = b"\xcb\x10\x6e\xca\x69\x45\xb7\x81\x1b\x57";
const BUCKETS_FILLED_18_OF_48: &[u8; 10] = b"\x08\x16\x8c\xb0\x65\xf5\x93\xbb\x88\xaf";
const BUCKETS_FILLED_23_OF_48: &[u8; 10] = b"\xb6\xe0\x71\xa6\x20\x1a\x6b\x2b\xe2\x44";
const BUCKETS_FILLED_24_OF_48: &[u8; 10] = b"\xd2\x21\x50\x57\xec\x82\x0b\xef\x36\xaa";
const BUCKETS_FILLED_25_OF_48: &[u8; 10] = b"\x6e\x24\x6e\xc2\x9b\x62\x19\x04\x13\xa0";
const BUCKETS_FILLED_32_OF_128: &[u8; 50] = b"\
    \xe3\x77\x84\x3a\xb1\x5e\x6b\x02\x50\x18\
    \x4b\x45\x23\x47\xe1\x1a\x90\x05\x3a\x29\
    \x7f\xcd\x05\xe2\xeb\xec\x44\x1f\xb5\xe8\
    \xe5\xb5\x7c\x3f\xff\x7f\x1d\x99\x05\xfb\
    \xc7\xca\xdf\x87\xed\x07\xff\x8b\xdb\xad";
const BUCKETS_FILLED_33_OF_128: &[u8; 50] = b"\
    \x45\x77\x4f\xfa\xe9\xc6\x83\xfe\x36\xee\
    \x63\x0a\x51\xa7\xcb\xa2\x24\x79\x39\xd5\
    \x9a\x7b\x52\x95\xf0\xc5\x29\xf2\x5f\x0b\
    \xd2\x28\xdd\x7e\xfe\xaf\xc0\x50\x86\xf5\
    \xf4\x3d\x4d\x0d\x2f\xc0\xd9\x57\xf5\x2a";
const BUCKETS_FILLED_64_OF_128: &[u8; 50] = b"\
    \xcc\x00\x19\xfe\xa3\x63\x1b\xe7\x6f\xf4\
    \x86\x7d\xfd\x06\xcd\xfc\x2a\x20\x6d\x61\
    \xe7\x88\xa8\x07\x96\x4d\xa0\x19\x01\x0b\
    \xa8\x4a\x2a\xd8\xbc\xad\xbe\xc6\x04\x50\
    \xb8\xbf\x65\xb6\x3f\x7d\xb4\x71\xee\x49";
const BUCKETS_FILLED_65_OF_128: &[u8; 50] = b"\
    \xa1\xdb\xcb\x51\x8c\x2c\x5d\xc9\x6a\x85\
    \x20\xcc\xad\x70\x47\xad\x3c\x18\x16\x7a\
    \xf5\xd5\xcc\xdd\x38\x3b\x24\xb4\x3d\x7f\
    \x1f\xc7\x3a\x8e\xbf\x27\xca\xcc\xb6\xc9\
    \x35\xc0\x58\xdc\x76\xd9\x4e\x31\xea\xb2";
const BUCKETS_FILLED_64_OF_256: &[u8; 50] = b"\
    \x30\x76\xaa\x04\x8b\x53\x71\xe3\x9a\x2d\
    \xcb\xb2\xd3\x0f\x9a\x2d\xcb\xb2\xd3\x0f\
    \x9a\x2d\xcb\xb2\xd3\x0f\x9a\x2d\xcb\xb2\
    \xd3\x0f\x9a\x2d\xcb\xb2\xd3\x0f\x9a\x2d\
    \xcb\xb2\xd3\x0f\x9a\x2d\xcb\xb2\xd3\x0f";
const BUCKETS_FILLED_65_OF_256: &[u8; 50] = b"\
    \x64\xe5\x33\xaa\x14\x82\x2a\x45\x82\x50\
    \xdc\x32\xd3\xd0\x53\xd6\x7c\x32\xd3\xd0\
    \x53\xd6\x7c\x32\xd3\xd0\x53\xd6\x7c\x32\
    \xd3\xd0\x53\xd6\x7c\x32\xd3\xd0\x53\xd6\
    \x7c\x32\xd3\xd0\x53\xd6\x7c\x32\xd3\xd0";
const BUCKETS_FILLED_128_OF_256: &[u8; 50] = b"\
    \xb8\x56\xea\xca\x15\xa2\x57\x23\xd2\x25\
    \xf1\x4c\x58\xd3\xca\x1a\x54\xf6\x09\x07\
    \xb0\x89\xce\xf1\x35\x3d\x25\xe4\xfc\x48\
    \xeb\xa1\xab\x49\xc8\x01\x67\x64\x93\x60\
    \xbb\xf3\x39\x98\xc0\xa9\x3e\x8b\x37\xce";
const BUCKETS_FILLED_129_OF_256: &[u8; 50] = b"\
    \x0d\x52\x88\x4e\xfc\x77\x3f\x47\x12\x8e\
    \x36\x30\x6f\x1a\x7d\x0b\x52\x1e\x98\x5c\
    \xc5\xa0\x1f\xb2\xa9\x43\xae\xe6\x4f\x69\
    \x61\x9e\xa3\xab\xdd\x2b\xe6\x60\x61\x5c\
    \x30\x47\xa4\x80\x7a\xde\x60\xb4\x7c\x26";

// 10-1 bytes, 30/48 buckets are filled (one of many perfect solutions and
// this data + '\0' is also a perfect solution with 36/48 buckets filled).
const STATISTICALLY_OKAY_WITH_LEN_9: &[u8; 9] = b"\x1d\x98\x29\x36\x25\xcb\xf5\xe2\x46";
// 50-1 bytes, 112/128 or 215/256 buckets are filled
// (statistically fine by itself on both normal and long variants)
const STATISTICALLY_OKAY_WITH_LEN_49: &[u8; 49] = b"\
    \x6c\xbc\x89\xe1\x61\x9e\x8e\
    \xeb\xcc\x8e\xbc\x2a\x17\x0b\
    \xe4\xcc\x25\xca\xf2\xe9\xe8\
    \x6e\xbc\x69\x25\x56\xb5\x5c\
    \xe5\x69\xf8\x48\x62\xf0\x00\
    \x97\xf0\xee\xad\x35\xc3\xed\
    \x41\xf6\x65\x8a\x02\x43\x37";

#[test]
fn prerequisites() {
    // Both WINDOW_SIZE and WINDOW_SIZE must fit in u32.
    assert!(u32::try_from(WINDOW_SIZE).is_ok());
    assert_ne!(WINDOW_SIZE, 0);
    // TAIL_SIZE must be greater than zero to encode
    // "we reached to 4GiB" condition by both len and tail_len.
    assert!(WINDOW_SIZE > 1);
}

#[test]
fn generator_options_compatibility() {
    let base_options = GeneratorOptions::new();
    let options = base_options.clone();
    assert!(options.is_tlsh_compatible());
    // Length processing mode is compatible with the official implementation.
    let options = base_options
        .clone()
        .length_processing_mode(DataLengthProcessingMode::Conservative);
    assert!(options.is_tlsh_compatible());
    let options = base_options
        .clone()
        .length_processing_mode(DataLengthProcessingMode::Optimistic);
    assert!(options.is_tlsh_compatible());
    // Setting incompatible options to false keeps the compatibility.
    let options = base_options.clone().allow_small_size_files(false);
    assert!(options.is_tlsh_compatible());
    let options = base_options
        .clone()
        .allow_statistically_weak_buckets_half(false);
    assert!(options.is_tlsh_compatible());
    let options = base_options
        .clone()
        .allow_statistically_weak_buckets_quarter(false);
    assert!(options.is_tlsh_compatible());
    let options = base_options.clone().pure_integer_qratio_computation(false);
    assert!(options.is_tlsh_compatible());
    // Incompatible with the official implementation:
    let options = base_options.clone().allow_small_size_files(true);
    assert!(!options.is_tlsh_compatible());
    let options = base_options
        .clone()
        .allow_statistically_weak_buckets_half(true);
    assert!(!options.is_tlsh_compatible());
    let options = base_options
        .clone()
        .allow_statistically_weak_buckets_quarter(true);
    assert!(!options.is_tlsh_compatible());
    // Compatible with the official implementation:
    let options = base_options.clone().pure_integer_qratio_computation(true);
    assert!(options.is_tlsh_compatible());
}

#[test]
fn tlsh_timing_unittest_vector() {
    // Displayed in the official implementation's timing_unittest.
    // Repeat 'A' through 'Z' for 1 000 000 bytes (except the last byte: '\0')
    let buffer: Vec<_> = (b'A'..=b'Z').cycle().take(1000000 - 1).chain([0]).collect();
    let mut gen = TlshGenerator::new();
    gen.update(&buffer);
    let hash = gen.finalize().unwrap();
    let expected = "T1A12500088C838B0A0F0EC3C0ACAB82F3B8228B0308CFA302338C0F0AE2C24F28000008";
    let expected = Tlsh::from_str(expected).unwrap();
    assert_eq!(hash, expected);
}

#[test]
fn tlsh_timing_unittest_vector_hidden() {
    // *not* displayed in the official implementation's timing_unittest
    // but used for comparison with another (above) and the expected value is
    // calculated using the official implementation.
    // Repeat 0x20, 0x21,.... (90 bytes) for 1 000 000 bytes (except the last byte: '\0')
    let buffer: Vec<_> = (b' '..(b' ' + 90))
        .cycle()
        .take(1000000 - 1)
        .chain([0])
        .collect();
    let mut gen = TlshGenerator::new();
    gen.update(&buffer);
    let hash = gen.finalize().unwrap();
    let expected = "T129251210F4C18D0A5F0661C4F64D905B585253A3024F022323E5074CC5601904886D1C";
    let expected = Tlsh::from_str(expected).unwrap();
    assert_eq!(hash, expected);
}

#[test]
fn generator_impls() {
    assert_eq!(TlshGenerator::new().inner, TlshGenerator::default().inner);
}

#[test]
fn generator_update_strategies() {
    type CustomGenerator = TlshGeneratorFor<hashes::Short>;
    let expected = hashes::Short::from_str("T1E16004017D3551777571D55C005CC5").unwrap();
    let buf = b"Hello, World!".as_slice();
    for divider1 in 0..=buf.len() {
        for divider2 in divider1..=buf.len() {
            let mut generator = CustomGenerator::new();
            generator.update(&buf[..divider1]);
            assert_eq!(generator.processed_len(), Some(divider1 as u32));
            generator.update(&buf[divider1..divider2]);
            assert_eq!(generator.processed_len(), Some(divider2 as u32));
            generator.update(&buf[divider2..]);
            assert_eq!(generator.processed_len(), Some(buf.len() as u32));
            assert_eq!(generator.finalize(), Ok(expected));
        }
    }
    // Update byte-to-byte
    {
        let mut generator = CustomGenerator::new();
        buf.iter().for_each(|&b| generator.update(&[b]));
        assert_eq!(generator.finalize(), Ok(expected));
    }
}

#[test]
fn generator_example_with_variants() {
    fn check_lorem_ipsum<F: ConstrainedFuzzyHashType + Debug>(expected: &str) {
        let expected = F::from_str(expected).unwrap();
        let mut generator = TlshGeneratorFor::<F>::new();
        generator.update(LOREM_IPSUM);
        let hash = generator.finalize().unwrap();
        assert_eq!(hash, expected);
    }
    check_lorem_ipsum::<hashes::Short>("T1E1F029B2FCAA4D5FE04846105FA5E2");
    check_lorem_ipsum::<hashes::Normal>(LOREM_IPSUM_HASH_NORMAL);
    check_lorem_ipsum::<hashes::NormalWithLongChecksum>(
        "T1DC33D4F0DC36520C1B007FD32079B226559FD998A0200725E75AFCEAC99F5881184A4B1AA2",
    );
    check_lorem_ipsum::<hashes::Long>(
        "T1DCF0DCA405C02AF1D4860CA5894A05301D60E9915198060A7044C608A1E89A11BD2B2836520C1B007FD32079B226559FD998A0200725E75AFCEAC99F5881184A4B1AA2"
    );
    check_lorem_ipsum::<hashes::LongWithLongChecksum>(
        "T1DC33D4F0DCA405C02AF1D4860CA5894A05301D60E9915198060A7044C608A1E89A11BD2B2836520C1B007FD32079B226559FD998A0200725E75AFCEAC99F5881184A4B1AA2"
    );
}

#[test]
fn min_lengths() {
    fn check<F: ConstrainedFuzzyHashType + Debug, const SIZE_BUCKETS: usize>(data: &[u8])
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        assert_eq!(SIZE_BUCKETS, F::NUMBER_OF_BUCKETS);
        // Construct the generator.
        // The input data is 1-byte less than the optimistic limit.
        let mut generator = TlshGeneratorFor::<F>::new();
        generator.update(data);
        let result = generator.finalize();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().category(),
            GeneratorErrorCategory::DataLength
        );
        // Now we get a valid fuzzy hash after appending a byte.
        // The input data is chosen not to cause statistic errors.
        generator.update(b"\0");
        let result = generator.finalize();
        assert!(result.is_ok());
        let size = (data.len() + 1) as u32;
        assert_eq!(LengthProcessingInfo::<SIZE_BUCKETS>::MIN, size);
        // Repeat until we reach the conservative minimum length,
        // make sure that finalization with conservative mode fails.
        for _ in size..LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE {
            let result = generator.finalize_with_options(
                GeneratorOptions::new()
                    .length_processing_mode(DataLengthProcessingMode::Conservative),
            );
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().category(),
                GeneratorErrorCategory::DataLength
            );
            generator.update(b"\0");
        }
        // We reached to the conservative minimum length.
        // Now we are able to construct a fuzzy hash in conservative mode.
        assert_eq!(
            generator.processed_len().unwrap(),
            LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE
        );
        let result = generator.finalize_with_options(
            GeneratorOptions::new().length_processing_mode(DataLengthProcessingMode::Conservative),
        );
        assert!(result.is_ok());
    }
    check::<hashes::Short, NUM_BUCKETS_SHORT>(STATISTICALLY_OKAY_WITH_LEN_9);
    check::<hashes::Normal, NUM_BUCKETS_NORMAL>(STATISTICALLY_OKAY_WITH_LEN_49);
    check::<hashes::NormalWithLongChecksum, NUM_BUCKETS_NORMAL>(STATISTICALLY_OKAY_WITH_LEN_49);
    check::<hashes::Long, NUM_BUCKETS_LONG>(STATISTICALLY_OKAY_WITH_LEN_49);
    check::<hashes::LongWithLongChecksum, NUM_BUCKETS_LONG>(STATISTICALLY_OKAY_WITH_LEN_49);
}

#[test]
fn max_lengths() {
    fn check<F: ConstrainedFuzzyHashType>()
    {
        assert_eq!(TlshGeneratorFor::<F>::MAX, crate::length::MAX);
    }
    check::<hashes::Short>();
    check::<hashes::Normal>();
    check::<hashes::NormalWithLongChecksum>();
    check::<hashes::Long>();
    check::<hashes::LongWithLongChecksum>();
}

/// Return the [`TlshGenerator`] which virtually processed
/// specified repetition of `[0xa4, 0x0e]`.
fn generator_with_a40e_repetitions(rep: u32) -> TlshGenerator {
    // Checksum repeats with period 141.
    const CHECKSUM_VALUES: [u8; 141] = [
        0x00, 0xde, 0xfc, 0xf4, 0x55, 0x24, 0xb0, 0x05, 0x0a, 0xd0, 0xa0, 0x64, 0x0d, 0xe8, 0x6f,
        0x81, 0xc4, 0xae, 0x01, 0xcb, 0xbe, 0x5d, 0xf5, 0x0e, 0x09, 0xcc, 0x67, 0x1e, 0x22, 0xc2,
        0xe0, 0xee, 0xc1, 0x8d, 0x12, 0xdf, 0xdb, 0x0b, 0x89, 0x1d, 0x8f, 0xab, 0x96, 0x3e, 0x19,
        0xe9, 0x04, 0x66, 0xf6, 0x9a, 0x8c, 0xbb, 0x4e, 0x25, 0x32, 0x5c, 0x20, 0xe7, 0xb1, 0x9f,
        0x6a, 0x62, 0xa9, 0x10, 0x39, 0xf9, 0x79, 0xf8, 0xa3, 0x7d, 0x84, 0x2a, 0xc5, 0xeb, 0x83,
        0xbd, 0xa6, 0xb2, 0x3c, 0x82, 0x2e, 0xca, 0x2f, 0xdc, 0xb8, 0x4b, 0x53, 0x9b, 0x40, 0x87,
        0xe4, 0x37, 0x49, 0x51, 0x80, 0x7c, 0x9e, 0x9c, 0xc7, 0x34, 0xa7, 0xfb, 0x58, 0x08, 0x54,
        0xf7, 0x59, 0xce, 0x43, 0x21, 0x65, 0x74, 0xd5, 0x1f, 0x7b, 0xec, 0xb6, 0xd8, 0xd2, 0x3d,
        0xc3, 0x6d, 0x2c, 0x23, 0xb3, 0x45, 0xea, 0x76, 0x71, 0x31, 0x07, 0xb7, 0x8a, 0xbc, 0x90,
        0x42, 0x94, 0xfa, 0x5a, 0xf0, 0xe6,
    ];
    assert!(rep <= 0x80000000);
    let total_size = 2u64 * rep as u64;
    // Note: heavily depends on the Generator internals
    let mut generator = TlshGenerator::new();
    if rep >= 2 {
        let rem_rep = rep - 2;
        generator.inner.tail = [0xa4, 0x0e, 0xa4, 0x0e];
        generator.inner.tail_len = 4;
        generator.inner.len = (total_size - 4) as u32;
        if rep > 2 {
            generator.inner.checksum =
                crate::hash::checksum::FuzzyHashChecksumData::<1, 128>::from_raw(&[
                    CHECKSUM_VALUES[rem_rep as usize % CHECKSUM_VALUES.len()],
                ]);
            generator.inner.buckets.buckets[0x14] = rem_rep;
            generator.inner.buckets.buckets[0x3d] = rem_rep;
            generator.inner.buckets.buckets[0x5b] = rem_rep.wrapping_mul(4);
            generator.inner.buckets.buckets[0x5c] = rem_rep;
            generator.inner.buckets.buckets[0x5d] = rem_rep;
            #[cfg(not(feature = "opt-low-memory-buckets"))]
            {
                generator.inner.buckets.buckets[0x8a] = rem_rep;
                generator.inner.buckets.buckets[0xaf] = rem_rep;
                generator.inner.buckets.buckets[0xe5] = rem_rep;
                generator.inner.buckets.buckets[0xf9] = rem_rep;
            }
        }
    } else if rep == 1 {
        generator.inner.tail = [0xa4, 0x0e, 0x00, 0x00];
        generator.inner.tail_len = 2;
    }
    generator
}

#[test]
fn test_generator_with_a40e_repetitions() {
    for rep in 0u32..8 {
        let generator = generator_with_a40e_repetitions(rep);
        let mut expected = TlshGenerator::new();
        for _ in 0..rep {
            expected.update(b"\xa4\x0e");
        }
        assert_eq!(generator.inner, expected.inner, "{rep}");
    }
}

#[test]
fn empty_data() {
    let generator = TlshGenerator::new();
    assert_eq!(generator.finalize(), Err(GeneratorError::TooSmallInput));
    assert_eq!(
        generator.finalize_with_options(GeneratorOptions::new().allow_small_size_files(true)),
        Err(GeneratorError::BucketsAreThreeQuarterEmpty)
    );
}

#[test]
fn large_data_examples() {
    let max_generator = generator_with_a40e_repetitions(0x80000000);
    let mut generator = generator_with_a40e_repetitions(0x7ffffffc);
    assert_eq!(generator.processed_len(), Some(0x7ffffffc * 2));
    // Attempt to finalize this would result in a "too large" error.
    assert_eq!(generator.finalize(), Err(GeneratorError::TooLargeInput));
    // Repeat 4 more times plus extra garbage
    // (ignored since it's past the first 4GiB).
    generator.update(b"\xa4\x0e\xa4\x0e\xa4\x0e\xa4\x0e\x01\x02");
    assert_eq!(max_generator.inner, generator.inner);
    assert_eq!(generator.processed_len(), None);
    // Feed more data (ignored)
    generator.update(b"\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c");
    assert_eq!(max_generator.inner, generator.inner);
    assert_eq!(generator.processed_len(), None);
}

#[test]
fn extreme_unbalanced_data_forced_to_finalize() {
    let mut generator = TlshGenerator::new();
    generator.update(BUCKETS_FILLED_32_OF_128);
    let options = GeneratorOptions::new()
        .allow_statistically_weak_buckets_quarter(true)
        .pure_integer_qratio_computation(true);
    let result = generator.finalize_with_options(options);
    assert_eq!(
        result.unwrap().to_string(),
        "T188904400C0C300300000C00000303C0000000C000300C00C00F30CC03F0C0000C30300"
    );
}

#[test]
fn min_nonzero_buckets_in_data() {
    fn check_state<F: ConstrainedFuzzyHashType>(data: &[u8], expected: usize) -> bool {
        let n_buckets = F::NUMBER_OF_BUCKETS;
        let mut generator = TlshGeneratorFor::<F>::new();
        generator.update(data);
        let result = generator.finalize();
        assert_eq!(generator.count_nonzero_buckets(), expected);
        if let Err(err) = result {
            assert_eq!(err.category(), GeneratorErrorCategory::DataDistribution);
            // More extreme distribution: BucketsAreThreeQuarterEmpty
            if expected <= n_buckets / 4 {
                assert_eq!(err, GeneratorError::BucketsAreThreeQuarterEmpty);
            }
        }
        result.is_ok()
    }
    // Short
    #[rustfmt::skip]
    fn check_short<F: ConstrainedFuzzyHashType>() {
        assert!(F::NUMBER_OF_BUCKETS == NUM_BUCKETS_SHORT);
        assert!(!check_state::<F>(BUCKETS_FILLED_12_OF_48, 12));
        assert!(!check_state::<F>(BUCKETS_FILLED_13_OF_48, 13));
        assert!(!check_state::<F>(BUCKETS_FILLED_17_OF_48, 17));
        assert!( check_state::<F>(BUCKETS_FILLED_18_OF_48, 18));
        assert!( check_state::<F>(BUCKETS_FILLED_23_OF_48, 23));
        assert!( check_state::<F>(BUCKETS_FILLED_24_OF_48, 24));
        assert!( check_state::<F>(BUCKETS_FILLED_25_OF_48, 25));
    }
    assert_eq!(
        FuzzyHashBucketsInfo::<NUM_BUCKETS_SHORT>::MIN_NONZERO_BUCKETS,
        18
    );
    check_short::<hashes::Short>();
    // Normal
    #[rustfmt::skip]
    fn check_normal<F: ConstrainedFuzzyHashType>() {
        assert!(F::NUMBER_OF_BUCKETS == NUM_BUCKETS_NORMAL);
        assert!(!check_state::<F>(BUCKETS_FILLED_32_OF_128, 32));
        assert!(!check_state::<F>(BUCKETS_FILLED_33_OF_128, 33));
        assert!(!check_state::<F>(BUCKETS_FILLED_64_OF_128, 64));
        assert!( check_state::<F>(BUCKETS_FILLED_65_OF_128, 65));
    }
    assert_eq!(
        FuzzyHashBucketsInfo::<NUM_BUCKETS_NORMAL>::MIN_NONZERO_BUCKETS,
        65
    );
    check_normal::<hashes::Normal>();
    check_normal::<hashes::NormalWithLongChecksum>();
    // Long
    #[rustfmt::skip]
    fn check_long<F: ConstrainedFuzzyHashType>() {
        assert!(F::NUMBER_OF_BUCKETS == NUM_BUCKETS_LONG);
        assert!(!check_state::<F>(BUCKETS_FILLED_64_OF_256, 64));
        assert!(!check_state::<F>(BUCKETS_FILLED_65_OF_256, 65));
        assert!(!check_state::<F>(BUCKETS_FILLED_128_OF_256, 128));
        assert!( check_state::<F>(BUCKETS_FILLED_129_OF_256, 129));
    }
    assert_eq!(
        FuzzyHashBucketsInfo::<NUM_BUCKETS_LONG>::MIN_NONZERO_BUCKETS,
        129
    );
    check_long::<hashes::Long>();
    check_long::<hashes::LongWithLongChecksum>();
}

#[test]
fn inevitable_unbalance_on_bucket_aggregation_example() {
    type Hash = hashes::Normal;
    let expected = "T11C90440000000000000000000000000000000000000000000000000000000000000000";
    let expected = Hash::from_str(expected).unwrap();
    let mut generator = TlshGeneratorFor::<Hash>::new();
    generator.update(
        b"\
        \x59\xc7\xb0\xe5\x47\xbe\x4c\x06\xdc\x95\x03\xc5\x16\x47\x2f\x8d\
        \x03\xea\x73\xd1\xc0\xb8\x79\xcd\x09\x87\xb9\x1f\xdf\xf9\x7c\xdb\
        \x38\x76\xd7\xf2\x04\xde\xc2\xcf\x9f\x7f\xab\xf0\xd5\x7a\x11\x56\
        \xf1\x89",
    );
    assert_eq!(generator.finalize().unwrap(), expected);
}
