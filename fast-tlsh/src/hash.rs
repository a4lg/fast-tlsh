// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The fuzzy hash and its parts (unless a part has its own module).

use core::fmt::Display;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::de::Visitor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::compare::ComparisonConfiguration;
use crate::errors::{OperationError, ParseError};
use crate::hash::body::FuzzyHashBody;
use crate::hash::checksum::FuzzyHashChecksum;
use crate::hash::qratios::FuzzyHashQRatios;
use crate::length::FuzzyHashLengthEncoding;
use crate::params::{ConstrainedFuzzyHashParams, FuzzyHashParams};
use crate::FuzzyHashType;

pub mod body;
pub mod checksum;
pub mod qratios;

/// Denotes the prefix on the TLSH's hexadecimal representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexStringPrefix {
    /// Raw / empty.
    ///
    /// If this value is specified to convert a fuzzy hash to string, no prefix
    /// is added and the result will purely consist of hexadecimal digits.
    ///
    /// If this value is specified to a parser method, it expects that the
    /// string immediately starts with a hexadecimal digit.
    Empty,

    /// With TLSH prefix and version (current default).
    ///
    /// This crate only supports version 1 prefix (`"T1"`).
    ///
    /// If this value is specified to convert a fuzzy hash to string, the TLSHv1
    /// prefix (`"T1"`) is added before the hexadecimal digits.
    ///
    /// If this value is specified to a parser method, it expects that the
    /// TLSHv1 prefix (`"T1"`) exists at the beginning.
    /// *This prefix is case-sensitive.*
    #[default]
    WithVersion,
}

/// The public part for later `pub use` at crate root.
pub(crate) mod public {
    use super::*;

    /// The trait to represent a fuzzy hash (TLSH).
    ///
    /// # TLSH Internals
    ///
    /// A fuzzy hash (TLSH) is composed of up to four parts:
    ///
    /// 1.  Checksum (checksum of the input, 1 or 3 bytes)
    ///     *   Trait: [`crate::hash::checksum::FuzzyHashChecksum`]
    ///     *   Internal Type: [`crate::hash::checksum::FuzzyHashChecksumData`]
    /// 2.  Data Length (approximated, encoded as an 8-bit integer)
    ///     *   Internal Type: [`crate::length::FuzzyHashLengthEncoding`]
    /// 3.  Q ratio pair, each Q ratio value reflecting the statistic distribution.
    ///     *   Internal Type: [`crate::hash::qratios::FuzzyHashQRatios`]
    /// 4.  Body.  Encoded as the specific number of quartile values
    ///     (each in 2-bits), in which the quartile count equals the number
    ///     of "buckets", used to gather statistic information (local features)
    ///     of the given input.
    ///     *   Trait: [`crate::hash::body::FuzzyHashBody`]
    ///     *   Internal Type: [`crate::hash::body::FuzzyHashBodyData`]
    ///
    /// Note that the checksum part can be always zero on some TLSH
    /// configurations (i.e. multi-threading is enabled or private flag is set).
    ///
    /// This trait is implemented by [`FuzzyHash`].
    pub trait FuzzyHashType: Sized + FromStr<Err = ParseError> + Display {
        /// The type of the checksum part.
        ///
        /// This is an instantiation of
        /// [`crate::hash::checksum::FuzzyHashChecksumData`].
        type ChecksumType: FuzzyHashChecksum;

        /// The type of the body part.
        ///
        /// This is an instantiation of
        /// [`crate::hash::body::FuzzyHashBodyData`].
        type BodyType: FuzzyHashBody;

        /// Number of the buckets.
        ///
        /// Specifically, this constant denotes the number of *effective*
        /// buckets that are used to construct a fuzzy hash.
        ///
        /// Sometimes, the number of physical buckets (number of possible
        /// results after [the Pearson hashing](crate::pearson) or its variant)
        /// differs from the number of effective buckets.
        ///
        /// | Variant | Effective Buckets | Physical Buckets |
        /// | ------- | -----------------:| ----------------:|
        /// | Short   |              `48` |            *`49` |
        /// | Normal  |             `128` |           *`256` |
        /// | Long    |             `256` |            `256` |
        ///
        /// On those cases, only the first effective buckets are used and the
        /// rest are ignored / dropped.
        const NUMBER_OF_BUCKETS: usize;

        /// Total size of the fuzzy hash (if represented as a byte array)
        /// in bytes (in the binary representation).
        ///
        /// This is the fixed size and required buffer size for the
        /// [`store_into_bytes()`](Self::store_into_bytes()) method.
        const SIZE_IN_BYTES: usize;

        /// Length in the hexadecimal string representation
        /// (except the prefix `"T1"`).
        ///
        /// This is always [`LEN_IN_STR`](Self::LEN_IN_STR) minus 2.
        ///
        /// This is the fixed size and required buffer size for the
        /// [`store_into_str_bytes()`](Self::store_into_str_bytes()) method with
        /// `prefix` of [`HexStringPrefix::Empty`].
        const LEN_IN_STR_EXCEPT_PREFIX: usize;

        /// Length in the hexadecimal string representation.
        ///
        /// This is always
        /// [`LEN_IN_STR_EXCEPT_PREFIX`](Self::LEN_IN_STR_EXCEPT_PREFIX) plus 2.
        ///
        /// This is the fixed size and required buffer size for the
        /// [`store_into_str_bytes()`](Self::store_into_str_bytes()) method with
        /// `prefix` of [`HexStringPrefix::WithVersion`].
        const LEN_IN_STR: usize;

        /// Returns the checksum part.
        fn checksum(&self) -> &Self::ChecksumType;
        /// Returns the length part.
        fn length(&self) -> &FuzzyHashLengthEncoding;
        /// Returns the Q ratio pair part.
        fn qratios(&self) -> &FuzzyHashQRatios;
        /// Returns the body part.
        fn body(&self) -> &Self::BodyType;

        /// Try parsing the fuzzy hash object from the given TLSH's hexadecimal
        /// representation and the operation mode.
        ///
        /// If the argument `prefix` is [`None`], the existence of the prefix
        /// will be auto-detected.  Otherwise, the existence of
        /// [the specified prefix](HexStringPrefix) is checked.
        fn from_str_bytes(
            bytes: &[u8],
            prefix: Option<HexStringPrefix>,
        ) -> Result<Self, ParseError>;

        /// Try parsing the fuzzy hash object from the given TLSH's hexadecimal
        /// representation and the operation mode.
        ///
        /// If the argument `prefix` is [`None`], the existence of the prefix
        /// will be auto-detected.  Otherwise, the existence of
        /// [the specified prefix](HexStringPrefix) is checked.
        #[inline(always)]
        fn from_str_with(s: &str, prefix: Option<HexStringPrefix>) -> Result<Self, ParseError> {
            Self::from_str_bytes(s.as_bytes(), prefix)
        }

        /// Store the contents of this object to the specified slice
        /// (in a binary format).
        ///
        /// This method stores the contents as a binary format suitable
        /// for serialization and parsing, to the specified slice.
        ///
        /// # The Binary Format with a Warning
        ///
        /// The binary format **slightly differs** from the representation you
        /// might expect from the TLSH's hexadecimal representation.
        ///
        /// The TLSH's hexadecimal representation has weird nibble endianness on
        /// the header (checksum, length and Q ratio pair parts).  For instance,
        /// the checksum part in the TLSH's hex representation `"42"` means
        /// the real checksum value of `0x24`.
        ///
        /// The body part is also *reversed* in a sense but this part is handled
        /// equivalently by this crate (because only "byte" ordering is
        /// reversed in one interpretation).  So, you may get the one you
        /// may expect from the TLSH's hexadecimal representation,
        /// at least in the body.
        ///
        /// The binary format used by this method doesn't do that conversion
        /// on the header.
        ///
        /// For instance, following TLSH hash
        /// ([normal 128 buckets with long 3-byte checksum](crate::hashes::NormalWithLongChecksum)):
        ///
        /// ```text
        /// T170F37CF0DC36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2 (raw)
        ///
        /// T1 70F37C F0 DC 36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2 (decomposed)
        /// |  |      |  |  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        /// |  |      |  |                Body / Buckets in quartiles (32-byte, 128 buckets)
        /// |  |      |  +- Q ratio pair (reversed; Q1 ratio -> Q2 ratio)
        /// |  |      +- Length (reversed)
        /// |  +- 3-byte Checksum (reversed per byte; AB CD EF -> BA DC FE)
        /// +- Header and version
        /// ```
        ///
        /// will be written as the following byte sequence by this method:
        ///
        /// ```text
        /// 073FC70FCD36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2 (raw)
        ///
        /// __ 073FC7 0F CD 36520C1B007FD320B9B266559FD998A0200725E75AFCEAC99F5881184A4B1AA2 (decomposed)
        /// |  |      |  |  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        /// |  |      |  |                Body / Buckets in quartiles (32-byte, 128 buckets)
        /// |  |      |  +- Q ratio pair (Q2 ratio -> Q1 ratio)                 (kept as is)
        /// |  |      +- Length
        /// |  +- 3-byte Checksum
        /// +- No header and version
        /// ```
        ///
        /// # The Specification
        ///
        /// This method concatenates:
        ///
        /// 1.  Checksum
        /// 2.  Length encoding
        /// 3.  Q ratio pair
        /// 4.  Body
        ///
        /// in that order (without any explicit variable length encodings or
        /// separators).  Each binary representation of the part can be
        /// retrieved as either an [`u8`] value or a slice / array of [`u8`].
        ///
        /// See [struct documentation](Self#tlsh-internals) for details.
        fn store_into_bytes(&self, out: &mut [u8]) -> Result<usize, OperationError>;

        /// Store the contents of this object to the specified slice
        /// (in the TLSH's hexadecimal representation).
        ///
        /// This method stores the contents as a TLSH's hexadecimal string
        /// representation with [the specified prefix](HexStringPrefix).
        fn store_into_str_bytes(
            &self,
            out: &mut [u8],
            prefix: HexStringPrefix,
        ) -> Result<usize, OperationError>;

        /// Compute the max distance on [comparison](Self::compare()) with
        /// the specified comparison configuration.
        ///
        /// If you need the maximum distance on the default configuration,
        /// use the first argument of [`Default::default()`].
        fn max_distance(config: ComparisonConfiguration) -> u32;

        /// Compare with another instance (with a configuration) and
        /// return the distance between them.
        ///
        /// Normally, you will likely use the default configuration and use
        /// [`compare()`](Self::compare()) instead.
        fn compare_with_config(&self, other: &Self, config: ComparisonConfiguration) -> u32;

        /// Compare with another instance with [the default configuration](ComparisonConfiguration::Default)
        /// and return the distance between them.
        ///
        /// If you need to use a non-default option, use
        /// [`compare_with_config()`](Self::compare_with_config()) instead.
        #[inline(always)]
        fn compare(&self, other: &Self) -> u32 {
            self.compare_with_config(other, ComparisonConfiguration::Default)
        }
    }
}

/// The inner representation and its implementation.
pub(crate) mod inner {
    use super::*;

    use crate::buckets::constrained::{FuzzyHashBucketMapper, FuzzyHashBucketsInfo};
    use crate::hash::body::FuzzyHashBodyData;
    use crate::hash::checksum::FuzzyHashChecksumData;
    use crate::macros::{invariant, optionally_unsafe};
    use crate::params::{ConstrainedVerboseFuzzyHashParams, VerboseFuzzyHashParams};
    #[cfg(not(feature = "opt-simd-convert-hex"))]
    use crate::parse::hex_str::encode_array;
    use crate::parse::hex_str::{encode_rev_1, encode_rev_array};

    /// The struct representing a fuzzy hash.
    ///
    /// This type is used as an inner representation of [`super::FuzzyHash`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FuzzyHash<
        const SIZE_CKSUM: usize,
        const SIZE_BODY: usize,
        const SIZE_BUCKETS: usize,
        const SIZE_IN_BYTES: usize,
        const SIZE_IN_STR_BYTES: usize,
    >
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        /// The body part.
        body: FuzzyHashBodyData<SIZE_BODY>,
        /// The checksum part.
        checksum: FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>,
        /// The encoded data length part.
        lvalue: FuzzyHashLengthEncoding,
        /// The Q ratio pair.
        qratios: FuzzyHashQRatios,
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        /// Creates an object from its raw parts.
        pub(crate) fn from_raw(
            body: FuzzyHashBodyData<SIZE_BODY>,
            checksum: FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>,
            lvalue: FuzzyHashLengthEncoding,
            qratios: FuzzyHashQRatios,
        ) -> Self {
            Self {
                body,
                checksum,
                lvalue,
                qratios,
            }
        }
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > FuzzyHashType
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type ChecksumType = FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>;
        type BodyType = FuzzyHashBodyData<SIZE_BODY>;

        const NUMBER_OF_BUCKETS: usize = SIZE_BUCKETS;
        const SIZE_IN_BYTES: usize = SIZE_IN_BYTES;
        const LEN_IN_STR_EXCEPT_PREFIX: usize = SIZE_IN_STR_BYTES - 2;
        const LEN_IN_STR: usize = SIZE_IN_STR_BYTES;

        #[inline]
        fn from_str_bytes(
            bytes: &[u8],
            prefix: Option<HexStringPrefix>,
        ) -> Result<Self, ParseError> {
            let mut bytes = bytes;
            let prefix = match prefix {
                None => {
                    if bytes.len() == Self::LEN_IN_STR_EXCEPT_PREFIX {
                        HexStringPrefix::Empty
                    } else if bytes.len() == Self::LEN_IN_STR {
                        HexStringPrefix::WithVersion
                    } else {
                        return Err(ParseError::InvalidStringLength);
                    }
                }
                Some(x) => x,
            };
            match prefix {
                HexStringPrefix::Empty => {
                    if bytes.len() != Self::LEN_IN_STR_EXCEPT_PREFIX {
                        return Err(ParseError::InvalidStringLength);
                    }
                }
                HexStringPrefix::WithVersion => {
                    if bytes.len() != Self::LEN_IN_STR {
                        return Err(ParseError::InvalidStringLength);
                    }
                    if &bytes[0..2] != b"T1" {
                        return Err(ParseError::InvalidPrefix);
                    }
                    bytes = &bytes[2..];
                }
            }
            let checksum = FuzzyHashChecksumData::<SIZE_CKSUM, SIZE_BUCKETS>::from_str_bytes(
                &bytes[0..SIZE_CKSUM * 2],
            )?;
            #[cfg(feature = "strict-parser")]
            if !checksum.is_valid() {
                return Err(ParseError::InvalidChecksum);
            }
            bytes = &bytes[SIZE_CKSUM * 2..];
            let lvalue = FuzzyHashLengthEncoding::from_str_bytes(&bytes[0..2])?;
            #[cfg(feature = "strict-parser")]
            if !lvalue.is_valid() {
                return Err(ParseError::LengthIsTooLarge);
            }
            let qratios = FuzzyHashQRatios::from_str_bytes(&bytes[2..4])?;
            let body = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(&bytes[4..])?;
            Ok(Self {
                body,
                checksum,
                lvalue,
                qratios,
            })
        }

        #[inline(always)]
        fn checksum(&self) -> &Self::ChecksumType {
            &self.checksum
        }
        #[inline(always)]
        fn length(&self) -> &FuzzyHashLengthEncoding {
            &self.lvalue
        }
        #[inline(always)]
        fn qratios(&self) -> &FuzzyHashQRatios {
            &self.qratios
        }
        #[inline(always)]
        fn body(&self) -> &Self::BodyType {
            &self.body
        }

        #[inline]
        fn store_into_bytes(&self, out: &mut [u8]) -> Result<usize, crate::errors::OperationError> {
            if out.len() < Self::SIZE_IN_BYTES {
                return Err(OperationError::BufferIsTooSmall);
            }
            out[0..SIZE_CKSUM].copy_from_slice(self.checksum.data());
            out[SIZE_CKSUM] = self.lvalue.value();
            out[SIZE_CKSUM + 1] = self.qratios.value();
            out[SIZE_CKSUM + 2..SIZE_IN_BYTES].copy_from_slice(self.body.data());
            Ok(Self::SIZE_IN_BYTES)
        }

        #[inline]
        fn store_into_str_bytes(
            &self,
            out: &mut [u8],
            prefix: HexStringPrefix,
        ) -> Result<usize, crate::errors::OperationError> {
            let len = match prefix {
                HexStringPrefix::Empty => Self::LEN_IN_STR_EXCEPT_PREFIX,
                HexStringPrefix::WithVersion => Self::LEN_IN_STR,
            };
            if out.len() < len {
                return Err(OperationError::BufferIsTooSmall);
            }
            let mut out = out;
            if prefix == HexStringPrefix::WithVersion {
                out[0..2].copy_from_slice(b"T1");
                out = &mut out[2..];
            }
            encode_rev_array(out, self.checksum.data());
            out = &mut out[SIZE_CKSUM * 2..];
            encode_rev_1(&mut out[0..2], self.lvalue.value());
            encode_rev_1(&mut out[2..4], self.qratios.value());
            cfg_if::cfg_if! {
                if #[cfg(feature = "opt-simd-convert-hex")] {
                    let _ = hex_simd::encode(
                        self.body.data(),
                        hex_simd::Out::from_slice(&mut out[4..]),
                        hex_simd::AsciiCase::Upper,
                    );
                } else {
                    encode_array(&mut out[4..], self.body.data());
                }
            }
            Ok(len)
        }

        #[inline]
        fn max_distance(config: ComparisonConfiguration) -> u32 {
            FuzzyHashBodyData::<SIZE_BODY>::MAX_DISTANCE
                + FuzzyHashChecksumData::<SIZE_CKSUM, SIZE_BUCKETS>::MAX_DISTANCE
                + FuzzyHashQRatios::MAX_DISTANCE
                + (match config {
                    ComparisonConfiguration::Default => FuzzyHashLengthEncoding::MAX_DISTANCE,
                    ComparisonConfiguration::NoDistance => 0,
                })
        }

        #[inline]
        fn compare_with_config(&self, other: &Self, config: ComparisonConfiguration) -> u32 {
            self.body.compare(&other.body)
                + self.checksum.compare(&other.checksum)
                + self.qratios.compare(&other.qratios)
                + (match config {
                    ComparisonConfiguration::Default => self.lvalue.compare(&other.lvalue),
                    ComparisonConfiguration::NoDistance => 0,
                })
        }
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > TryFrom<&[u8; SIZE_IN_BYTES]>
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type Error = ParseError;

        #[inline]
        fn try_from(value: &[u8; SIZE_IN_BYTES]) -> Result<Self, Self::Error> {
            let checksum = FuzzyHashChecksumData::<SIZE_CKSUM, SIZE_BUCKETS>::from_raw(
                value[0..SIZE_CKSUM].try_into().unwrap(),
            );
            #[cfg(feature = "strict-parser")]
            if !checksum.is_valid() {
                return Err(ParseError::InvalidChecksum);
            }
            let lvalue = FuzzyHashLengthEncoding::from_raw(value[SIZE_CKSUM]);
            #[cfg(feature = "strict-parser")]
            if !lvalue.is_valid() {
                return Err(ParseError::LengthIsTooLarge);
            }
            let qratios = FuzzyHashQRatios::from_raw(value[SIZE_CKSUM + 1]);
            let value = &value[SIZE_CKSUM + 2..];
            optionally_unsafe! {
                invariant!(value.len() == SIZE_BODY);
            }
            Ok(Self {
                checksum,
                lvalue,
                qratios,
                body: FuzzyHashBodyData::from_raw(value.try_into().unwrap()),
            })
        }
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > TryFrom<&[u8]>
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type Error = ParseError;

        #[inline(always)]
        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            if value.len() != SIZE_IN_BYTES {
                return Err(ParseError::InvalidStringLength);
            }
            let value: &[u8; SIZE_IN_BYTES] = value.try_into().unwrap();
            Self::try_from(value)
        }
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > FromStr
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type Err = ParseError;
        #[inline(always)]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::from_str_with(s, None)
        }
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Display
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut buf = [0u8; SIZE_IN_STR_BYTES];
            self.store_into_str_bytes(&mut buf, HexStringPrefix::WithVersion)
                .unwrap();
            cfg_if::cfg_if! {
                if #[cfg(feature = "unsafe")] {
                    unsafe {
                        f.write_str(core::str::from_utf8_unchecked(&buf))
                    }
                } else {
                    f.write_str(core::str::from_utf8(&buf).unwrap())
                }
            }
        }
    }

    #[cfg(feature = "serde")]
    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Serialize
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            if serializer.is_human_readable() {
                let mut buffer = [0u8; SIZE_IN_STR_BYTES];
                self.store_into_str_bytes(&mut buffer, HexStringPrefix::WithVersion)
                    .unwrap();
                #[cfg(feature = "unsafe")]
                unsafe {
                    serializer.serialize_str(core::str::from_utf8_unchecked(&buffer))
                }
                #[cfg(not(feature = "unsafe"))]
                {
                    serializer.serialize_str(core::str::from_utf8(&buffer).unwrap())
                }
            } else {
                let mut buffer = [0u8; SIZE_IN_BYTES];
                self.store_into_bytes(&mut buffer).unwrap();
                serializer.serialize_bytes(&buffer)
            }
        }
    }

    /// The visitor struct to handle [fuzzy hash](FuzzyHash) deserialization
    /// as either a string or a sequence of bytes.
    ///
    /// The corresponding visitor implementation handles a fuzzy hash as
    /// either a string or a sequence of bytes, both representing the string
    /// representation of that fuzzy hash.
    ///
    /// This visitor is used on human-readable formats (such as JSON).
    #[cfg(feature = "serde")]
    struct FuzzyHashStringVisitor<
        const SIZE_CKSUM: usize,
        const SIZE_BODY: usize,
        const SIZE_BUCKETS: usize,
        const SIZE_IN_BYTES: usize,
        const SIZE_IN_STR_BYTES: usize,
    >
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams;

    #[cfg(feature = "serde")]
    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Visitor<'_>
        for FuzzyHashStringVisitor<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type Value =
            FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("a fuzzy hash string")
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_bytes(v.as_bytes())
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Self::Value::from_str_bytes(v, None).map_err(serde::de::Error::custom::<ParseError>)
        }
    }

    /// The visitor struct to handle [fuzzy hash](FuzzyHash) deserialization
    /// as a byte sequence.
    ///
    /// The corresponding visitor implementation handles a fuzzy hash as
    /// a plain sequence of bytes.
    ///
    /// This visitor is used on machine-friendly formats (such as Postcard).
    #[cfg(feature = "serde")]
    struct FuzzyHashBytesVisitor<
        const SIZE_CKSUM: usize,
        const SIZE_BODY: usize,
        const SIZE_BUCKETS: usize,
        const SIZE_IN_BYTES: usize,
        const SIZE_IN_STR_BYTES: usize,
    >
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams;

    #[cfg(feature = "serde")]
    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Visitor<'_>
        for FuzzyHashBytesVisitor<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        type Value =
            FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str("struct FuzzyHash")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v.len() != SIZE_IN_BYTES {
                return Err(serde::de::Error::invalid_length(v.len(), &self));
            }
            Ok(Self::Value::try_from(v).unwrap())
        }
    }

    #[cfg(feature = "serde")]
    impl<
            'de,
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Deserialize<'de>
        for FuzzyHash<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            if deserializer.is_human_readable() {
                #[cfg(feature = "serde-buffered")]
                {
                    deserializer.deserialize_string(
                        FuzzyHashStringVisitor::<
                            SIZE_CKSUM,
                            SIZE_BODY,
                            SIZE_BUCKETS,
                            SIZE_IN_BYTES,
                            SIZE_IN_STR_BYTES,
                        >,
                    )
                }
                #[cfg(not(feature = "serde-buffered"))]
                {
                    deserializer.deserialize_str(
                        FuzzyHashStringVisitor::<
                            SIZE_CKSUM,
                            SIZE_BODY,
                            SIZE_BUCKETS,
                            SIZE_IN_BYTES,
                            SIZE_IN_STR_BYTES,
                        >,
                    )
                }
            } else {
                #[cfg(feature = "serde-buffered")]
                {
                    deserializer.deserialize_byte_buf(
                        FuzzyHashBytesVisitor::<
                            SIZE_CKSUM,
                            SIZE_BODY,
                            SIZE_BUCKETS,
                            SIZE_IN_BYTES,
                            SIZE_IN_STR_BYTES,
                        >,
                    )
                }
                #[cfg(not(feature = "serde-buffered"))]
                {
                    deserializer.deserialize_bytes(
                        FuzzyHashBytesVisitor::<
                            SIZE_CKSUM,
                            SIZE_BODY,
                            SIZE_BUCKETS,
                            SIZE_IN_BYTES,
                            SIZE_IN_STR_BYTES,
                        >,
                    )
                }
            }
        }
    }
}

/// Type macro to represent the inner hash type of [`FuzzyHash`]
/// (an instantiation of [`inner::FuzzyHash`]).
macro_rules! inner_type {
    ($size_checksum:expr, $size_buckets:expr) => {
        <FuzzyHashParams<{$size_checksum}, {$size_buckets}> as ConstrainedFuzzyHashParams>::InnerFuzzyHashType
    };
}

/// The fuzzy hash struct representing a fuzzy hash (TLSH).
///
/// For the main functionalities, see [`FuzzyHashType`] documentation.
///
/// This struct supports conversion from:
///
/// *   An array of [`u8`]  
///     (containing a binary representation as described in
///     [`FuzzyHashType::store_into_bytes()`]) with the length
///     [`SIZE_IN_BYTES`](Self::SIZE_IN_BYTES) (through [`TryFrom`]),
/// *   A slice of [`u8`]  
///     (containing a binary representation as described in
///     [`FuzzyHashType::store_into_bytes()`]) with the length
///     [`SIZE_IN_BYTES`](Self::SIZE_IN_BYTES) (through [`TryFrom`]), or
/// *   A string  
///     with the TLSH hexadecimal representation (through [`FromStr`]).
///
/// and to:
///
/// *   A slice of [`u8`]  
///     containing a binary representation
///     using [`FuzzyHashType::store_into_bytes()`] or
/// *   A string (a slice of [`u8`] or a [`String`])  
///     with the TLSH hexadecimal representation
///     using either [`FuzzyHashType::store_into_str_bytes()`] or
///     through the [`Display`]-based formatting (including [`ToString`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyHash<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
{
    /// The inner object representing actual contents of the fuzzy hash.
    inner: inner_type!(SIZE_CKSUM, SIZE_BUCKETS),
}

impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
{
    /// Creates an object from the inner object.
    #[inline(always)]
    pub(crate) fn new(inner: inner_type!(SIZE_CKSUM, SIZE_BUCKETS)) -> Self {
        Self { inner }
    }
}

impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> crate::FuzzyHashType
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
{
    type ChecksumType = <inner_type!(SIZE_CKSUM, SIZE_BUCKETS) as FuzzyHashType>::ChecksumType;
    type BodyType = <inner_type!(SIZE_CKSUM, SIZE_BUCKETS) as FuzzyHashType>::BodyType;

    const NUMBER_OF_BUCKETS: usize = <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::NUMBER_OF_BUCKETS;
    const SIZE_IN_BYTES: usize = <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::SIZE_IN_BYTES;
    const LEN_IN_STR_EXCEPT_PREFIX: usize =
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::LEN_IN_STR_EXCEPT_PREFIX;
    const LEN_IN_STR: usize = <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::LEN_IN_STR;
    #[inline(always)]
    fn from_str_bytes(
        bytes: &[u8],
        prefix: Option<HexStringPrefix>,
    ) -> Result<Self, crate::errors::ParseError> {
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::from_str_bytes(bytes, prefix)
            .map(|inner| Self { inner })
    }
    #[inline(always)]
    fn checksum(&self) -> &Self::ChecksumType {
        self.inner.checksum()
    }
    #[inline(always)]
    fn length(&self) -> &FuzzyHashLengthEncoding {
        self.inner.length()
    }
    #[inline(always)]
    fn qratios(&self) -> &FuzzyHashQRatios {
        self.inner.qratios()
    }
    #[inline(always)]
    fn body(&self) -> &Self::BodyType {
        self.inner.body()
    }
    #[inline(always)]
    fn store_into_bytes(&self, out: &mut [u8]) -> Result<usize, crate::errors::OperationError> {
        self.inner.store_into_bytes(out)
    }
    #[inline(always)]
    fn store_into_str_bytes(
        &self,
        out: &mut [u8],
        prefix: HexStringPrefix,
    ) -> Result<usize, OperationError> {
        self.inner.store_into_str_bytes(out, prefix)
    }
    #[inline(always)]
    fn max_distance(config: ComparisonConfiguration) -> u32 {
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::max_distance(config)
    }
    #[inline(always)]
    fn compare_with_config(&self, other: &Self, config: ComparisonConfiguration) -> u32 {
        self.inner.compare_with_config(&other.inner, config)
    }
}
impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> Display
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
{
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}
impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> FromStr
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
    inner_type!(SIZE_CKSUM, SIZE_BUCKETS): FromStr<Err = ParseError>,
{
    type Err = ParseError;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::from_str(s).map(Self::new)
    }
}
impl<'a, const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize, const SIZE_IN_BYTES: usize>
    TryFrom<&'a [u8; SIZE_IN_BYTES]> for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
    inner_type!(SIZE_CKSUM, SIZE_BUCKETS): TryFrom<&'a [u8; SIZE_IN_BYTES], Error = ParseError>,
{
    type Error = ParseError;
    #[inline(always)]
    fn try_from(value: &'a [u8; SIZE_IN_BYTES]) -> Result<Self, Self::Error> {
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::try_from(value).map(Self::new)
    }
}
impl<'a, const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> TryFrom<&'a [u8]>
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
    inner_type!(SIZE_CKSUM, SIZE_BUCKETS): TryFrom<&'a [u8], Error = ParseError>,
{
    type Error = ParseError;
    #[inline(always)]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS)>::try_from(value).map(Self::new)
    }
}
#[cfg(feature = "serde")]
impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> Serialize
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
    inner_type!(SIZE_CKSUM, SIZE_BUCKETS): Serialize,
{
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Wrap inner implementation
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS) as Serialize>::serialize(&self.inner, serializer)
    }
}
#[cfg(feature = "serde")]
impl<'de, const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize> Deserialize<'de>
    for FuzzyHash<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashParams<SIZE_CKSUM, SIZE_BUCKETS>: ConstrainedFuzzyHashParams,
    inner_type!(SIZE_CKSUM, SIZE_BUCKETS): Deserialize<'de>,
{
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Wrap inner implementation
        <inner_type!(SIZE_CKSUM, SIZE_BUCKETS) as Deserialize<'de>>::deserialize(deserializer)
            .map(Self::new)
    }
}

mod tests;
