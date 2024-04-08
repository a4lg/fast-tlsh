// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::hash`].

#![cfg(test)]

use super::{ComparisonConfiguration, HexStringPrefix};

use core::str::FromStr;

use crate::buckets::NUM_BUCKETS_SHORT;
use crate::errors::{OperationError, ParseError};
use crate::hashes;
use crate::length::ENCODED_VALUE_SIZE;
use crate::FuzzyHashType;

#[test]
fn hex_string_prefix_default() {
    // Check its default value.
    assert_eq!(
        <HexStringPrefix as Default>::default(),
        HexStringPrefix::WithVersion
    );
}

#[test]
fn comparison_configuration_default() {
    // Check its default value.
    assert_eq!(
        <ComparisonConfiguration as Default>::default(),
        ComparisonConfiguration::Default
    );
}

#[test]
fn from_and_to_str() {
    const HASH_STR: &str = "T1E16004017D3551777571D55C005CC5";
    type CustomTlsh = hashes::Short;
    let str = CustomTlsh::from_str(HASH_STR).unwrap().to_string();
    assert_eq!(str.as_str(), HASH_STR);
}

#[test]
fn from_str_prefix() {
    const HASH_STR_0: &str = "E16004017D3551777571D55C005CC5";
    const HASH_STR_1: &str = "T1E16004017D3551777571D55C005CC5";
    type CustomTlsh = hashes::Short;
    // Auto detection
    let hash0 = CustomTlsh::from_str_with(HASH_STR_0, None);
    let hash1 = CustomTlsh::from_str_with(HASH_STR_1, None);
    let hash = hash0;
    assert!(hash.is_ok());
    assert_eq!(hash0, hash1);
    // Explicit prefix
    let hash0 = CustomTlsh::from_str_with(HASH_STR_0, Some(HexStringPrefix::Empty));
    let hash1 = CustomTlsh::from_str_with(HASH_STR_1, Some(HexStringPrefix::WithVersion));
    assert_eq!(hash, hash0);
    assert_eq!(hash, hash1);
    // Explicit prefix (wrong mode)
    let hash0 = CustomTlsh::from_str_with(HASH_STR_0, Some(HexStringPrefix::WithVersion));
    let hash1 = CustomTlsh::from_str_with(HASH_STR_1, Some(HexStringPrefix::Empty));
    assert_eq!(hash0, Err(ParseError::InvalidStringLength));
    assert_eq!(hash1, Err(ParseError::InvalidStringLength));
}

#[test]
fn from_str_other_errors() {
    type CustomTlsh = hashes::Short;
    // Empty string (does not match to valid string length)
    assert_eq!(
        CustomTlsh::from_str(""),
        Err(ParseError::InvalidStringLength)
    );
    // TNULL (does not match to valid string length)
    assert_eq!(
        CustomTlsh::from_str("TNULL"),
        Err(ParseError::InvalidStringLength)
    );
    // Invalid prefix
    assert_eq!(
        CustomTlsh::from_str("T2E16004017D3551777571D55C005CC5"),
        Err(ParseError::InvalidPrefix)
    );
    // Invalid checksum ('@' is not valid)
    assert_eq!(
        CustomTlsh::from_str("T1E@6004017D3551777571D55C005CC5"),
        Err(ParseError::InvalidCharacter)
    );
    // Invalid length ('@' is not valid)
    assert_eq!(
        CustomTlsh::from_str("T1E16@04017D3551777571D55C005CC5"),
        Err(ParseError::InvalidCharacter)
    );
    // Invalid Q ratios ('@' is not valid)
    assert_eq!(
        CustomTlsh::from_str("T1E1600@017D3551777571D55C005CC5"),
        Err(ParseError::InvalidCharacter)
    );
    // Invalid body ('@' is not valid)
    assert_eq!(
        CustomTlsh::from_str("T1E16004@17D3551777571D55C005CC5"),
        Err(ParseError::InvalidCharacter)
    );
}

#[test]
fn try_from_bytes() {
    type CustomTlsh = hashes::Short;
    // Reference data
    let reference = CustomTlsh::from_str("T1E16004017D3551777571D55C005CC5");
    assert!(reference.is_ok());
    assert_eq!(
        CustomTlsh::try_from(b"\x1e\x06\x40\x01\x7d\x35\x51\x77\x75\x71\xd5\x5c\x00\x5c\xc5"),
        reference
    );
    assert_eq!(
        CustomTlsh::try_from(
            b"\x1e\x06\x40\x01\x7d\x35\x51\x77\x75\x71\xd5\x5c\x00\x5c\xc5" as &[u8]
        ),
        reference
    );
    // Empty data (wrong length)
    assert_eq!(
        CustomTlsh::try_from(b"" as &[u8]),
        Err(ParseError::InvalidStringLength)
    );
}

#[test]
fn strict_parser_str_length() {
    assert_eq!(ENCODED_VALUE_SIZE, 0xaa);
    const STR1: &str = "T14D9ADDD869983B33E27B4F308C459ED4F77FE24A4BC42C52CF1C9F046D5945AEA69888";
    const STR2: &str = "T14DAADDD869983B33E27B4F308C459ED4F77FE24A4BC42C52CF1C9F046D5945AEA69888";
    // Length encoding: 0xa9 (the maximum valid encoding)
    let result = hashes::Normal::from_str(STR1);
    assert!(result.is_ok());
    // Length encoding: 0xaa (invalid encoding)
    let result = hashes::Normal::from_str(STR2);
    cfg_if::cfg_if! {
        if #[cfg(feature = "strict-parser")] {
            assert_eq!(result, Err(ParseError::LengthIsTooLarge));
        } else {
            assert!(result.is_ok()); // Accepted by default
        }
    }
}

#[test]
fn strict_parser_str_checksum() {
    assert_eq!(NUM_BUCKETS_SHORT, 0x30);
    const STR1: &str = "T103D0BA38361825F4FA6D0B575C1CB5";
    const STR2: &str = "T113D0BA38361825F4FA6D0B575C1CB5";
    // Checksum: 0x30 (the maximum valid value)
    let result = hashes::Short::from_str(STR1);
    assert!(result.is_ok());
    // Checksum: 0x31 (invalid checksum)
    let result = hashes::Short::from_str(STR2);
    cfg_if::cfg_if! {
        if #[cfg(feature = "strict-parser")] {
            assert_eq!(result, Err(ParseError::InvalidChecksum));
        } else {
            assert!(result.is_ok()); // Accepted by default
        }
    }
}

#[test]
fn strict_parser_bytes_length() {
    // Corresponds: strict_parser_str_length
    assert_eq!(ENCODED_VALUE_SIZE, 0xAA);
    const BYTES1: &[u8] = b"\xD4\xA9\xDD\
        \xD8\x69\x98\x3B\x33\xE2\x7B\x4F\x30\x8C\x45\x9E\xD4\xF7\x7F\xE2\x4A\x4B\xC4\x2C\x52\xCF\x1C\x9F\x04\x6D\x59\x45\xAE\xA6\x98\x88";
    const BYTES2: &[u8] = b"\xD4\xAA\xDD\
        \xD8\x69\x98\x3B\x33\xE2\x7B\x4F\x30\x8C\x45\x9E\xD4\xF7\x7F\xE2\x4A\x4B\xC4\x2C\x52\xCF\x1C\x9F\x04\x6D\x59\x45\xAE\xA6\x98\x88";
    // Length encoding: 0xa9 (the maximum valid encoding)
    let result = hashes::Normal::try_from(BYTES1);
    assert!(result.is_ok());
    // Length encoding: 0xaa (invalid encoding)
    let result = hashes::Normal::try_from(BYTES2);
    cfg_if::cfg_if! {
        if #[cfg(feature = "strict-parser")] {
            assert_eq!(result, Err(ParseError::LengthIsTooLarge));
        } else {
            assert!(result.is_ok()); // Accepted by default
        }
    }
}

#[test]
fn strict_parser_bytes_checksum() {
    // Corresponds: strict_parser_str_checksum
    assert_eq!(NUM_BUCKETS_SHORT, 0x30);
    const BYTES1: &[u8] = b"\x30\x0D\xAB\
        \x38\x36\x18\x25\xF4\xFA\x6D\x0B\x57\x5C\x1C\xB5";
    const BYTES2: &[u8] = b"\x31\x0D\xAB\
        \x38\x36\x18\x25\xF4\xFA\x6D\x0B\x57\x5C\x1C\xB5";
    // Checksum: 0x30 (the maximum valid value)
    let result = hashes::Short::try_from(BYTES1);
    assert!(result.is_ok());
    // Checksum: 0x31 (invalid checksum)
    let result = hashes::Short::try_from(BYTES2);
    cfg_if::cfg_if! {
        if #[cfg(feature = "strict-parser")] {
            assert_eq!(result, Err(ParseError::InvalidChecksum));
        } else {
            assert!(result.is_ok()); // Accepted by default
        }
    }
}

#[test]
fn internal_data() {
    type CustomTlsh = hashes::Short;
    let hash = CustomTlsh::from_str("T1E16004017D3551777571D55C005CC5").unwrap();
    assert_eq!(hash.checksum().data(), b"\x1e");
    assert_eq!(hash.length().value(), 0x06);
    assert_eq!(hash.qratios().q1ratio(), 0x0);
    assert_eq!(hash.qratios().q2ratio(), 0x4);
    assert_eq!(
        hash.body().data().as_slice(),
        b"\x01\x7d\x35\x51\x77\x75\x71\xd5\x5c\x00\x5c\xc5"
    );
}

#[test]
fn from_and_to_str_prefix() {
    const HASH_STR_0: &str = "E16004017D3551777571D55C005CC5";
    const HASH_STR_1: &str = "T1E16004017D3551777571D55C005CC5";
    type CustomTlsh = hashes::Short;
    let mut buffer = [0u8; CustomTlsh::LEN_IN_STR];
    let hash0 = CustomTlsh::from_str(HASH_STR_0).unwrap();
    let hash1 = CustomTlsh::from_str(HASH_STR_1).unwrap();
    assert_eq!(hash0, hash1);
    // Of course, we can strip/append prefix using store_into_str_bytes.
    let size = hash0
        .store_into_str_bytes(buffer.as_mut_slice(), HexStringPrefix::WithVersion)
        .unwrap();
    assert_eq!(&buffer[..size], HASH_STR_1.as_bytes());
    let size = hash1
        .store_into_str_bytes(buffer.as_mut_slice(), HexStringPrefix::Empty)
        .unwrap();
    assert_eq!(&buffer[..size], HASH_STR_0.as_bytes());
}

#[test]
fn store_into_bytes_example() {
    type CustomTlsh = hashes::NormalWithLongChecksum;
    // In the example: 073FC70FCD36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2
    const BYTES_REPRESENTATION: &[u8] = b"\
        \x07\x3F\xC7\
        \x0F\
        \xCD\
        \x36\x52\x0C\x1B\x00\x7F\xD3\x20\
        \xB9\xB2\x66\x55\x9F\xD9\x98\xA0\
        \x20\x07\x25\xE7\x5A\xFC\xEA\xC9\
        \x9F\x58\x81\x18\x4A\x4B\x1A\xA2";
    // In the example: T170F37CF0DC36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2
    let hash = CustomTlsh::from_str(
        "T1\
        70F37C\
        F0\
        DC\
        36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2",
    )
    .unwrap();
    let mut buffer = [0; CustomTlsh::SIZE_IN_BYTES];
    assert_eq!(
        hash.store_into_bytes(&mut buffer),
        Ok(CustomTlsh::SIZE_IN_BYTES)
    );
    assert_eq!(&buffer, BYTES_REPRESENTATION);
    let hash2 = CustomTlsh::try_from(&buffer).unwrap();
    let hash3 = CustomTlsh::try_from(BYTES_REPRESENTATION).unwrap();
    assert_eq!(hash, hash2);
    assert_eq!(hash, hash3);
}

#[test]
fn store_into_bytes_insufficient_buffer() {
    let hash = hashes::Normal::from_str(
        "T14D9ADDD869983B33E27B4F308C459ED4F77FE24A4BC42C52CF1C9F046D5945AEA69888",
    )
    .unwrap();
    let mut buffer = [];
    assert_eq!(
        hash.store_into_bytes(buffer.as_mut_slice()),
        Err(OperationError::BufferIsTooSmall)
    );
}

#[test]
fn store_into_str_bytes_insufficient_buffer() {
    let hash = hashes::Normal::from_str(
        "T14D9ADDD869983B33E27B4F308C459ED4F77FE24A4BC42C52CF1C9F046D5945AEA69888",
    )
    .unwrap();
    let mut buffer = [];
    assert_eq!(
        hash.store_into_str_bytes(buffer.as_mut_slice(), HexStringPrefix::Empty),
        Err(OperationError::BufferIsTooSmall)
    );
    assert_eq!(
        hash.store_into_str_bytes(buffer.as_mut_slice(), HexStringPrefix::WithVersion),
        Err(OperationError::BufferIsTooSmall)
    );
}

#[test]
fn test_compare_with_config() {
    let hash1 = hashes::Normal::from_str(
        "T11632623FBA48037706C20162BB9764CBF21E903F3B552568354CC1681F6BA6543FB6EA",
    )
    .unwrap();
    let hash2 = hashes::Normal::from_str(
        "T11642623FBA48037706C20162BB9764CBF21E903F3B552568354CC1681F6BA6543FB6EA",
    )
    .unwrap();
    // Compare each parts
    assert_eq!(hash1.checksum().data(), hash2.checksum().data());
    assert_ne!(hash1.length(), hash2.length());
    assert_eq!(hash1.qratios(), hash2.qratios());
    assert_eq!(hash1.body().data(), hash2.body().data());
    // Comparison with compare and compare_with_config
    assert_eq!(hash1.compare(&hash2), 1);
    assert_eq!(
        hash1.compare_with_config(&hash2, ComparisonConfiguration::Default),
        1
    );
    assert_eq!(
        hash1.compare_with_config(&hash2, ComparisonConfiguration::NoDistance),
        0
    );
}

#[test]
fn clear_checksum_modification() {
    const HASH_STR_1: &str = "T1E16004017D3551777571D55C005CC5";
    const HASH_STR_2: &str = "T1006004017D3551777571D55C005CC5";
    type CustomTlsh = hashes::Short;
    let hash_1 = CustomTlsh::from_str(HASH_STR_1).unwrap();
    let hash_2 = CustomTlsh::from_str(HASH_STR_2).unwrap();
    // Because hash2 has cleared checksum, they are different.
    assert_ne!(hash_1, hash_2);
    // Clearing the checksum part should succeed on our fuzzy hash type.
    let mut hash_1 = hash_1;
    hash_1.clear_checksum();
    // After clearing the checksum, they should match.
    assert_eq!(hash_1, hash_2);
}

#[test]
fn max_distances() {
    // Compare with pre-computed values.
    assert_eq!(
        hashes::Short::max_distance(ComparisonConfiguration::NoDistance),
        457
    );
    assert_eq!(
        hashes::Short::max_distance(ComparisonConfiguration::Default),
        457 + 1536
    );
    assert_eq!(
        hashes::Normal::max_distance(ComparisonConfiguration::NoDistance),
        937
    );
    assert_eq!(
        hashes::Normal::max_distance(ComparisonConfiguration::Default),
        937 + 1536
    );
    assert_eq!(
        hashes::NormalWithLongChecksum::max_distance(ComparisonConfiguration::NoDistance),
        939
    );
    assert_eq!(
        hashes::NormalWithLongChecksum::max_distance(ComparisonConfiguration::Default),
        939 + 1536
    );
    assert_eq!(
        hashes::Long::max_distance(ComparisonConfiguration::NoDistance),
        1705
    );
    assert_eq!(
        hashes::Long::max_distance(ComparisonConfiguration::Default),
        1705 + 1536
    );
    assert_eq!(
        hashes::LongWithLongChecksum::max_distance(ComparisonConfiguration::NoDistance),
        1707
    );
    assert_eq!(
        hashes::LongWithLongChecksum::max_distance(ComparisonConfiguration::Default),
        1707 + 1536
    );
}
