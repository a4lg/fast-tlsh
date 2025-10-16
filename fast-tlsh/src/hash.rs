// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The fuzzy hash and its parts (unless a part has its own module).

use core::fmt::Display;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::internals::compare::ComparisonConfiguration;
use crate::internals::errors::{OperationError, ParseError};
use crate::internals::hash::qratios::FuzzyHashQRatios;
use crate::internals::hash::{FuzzyHashType, HexStringPrefix};
use crate::internals::length::FuzzyHashLengthEncoding;
use crate::internals::params::{ConstrainedFuzzyHashParams, FuzzyHashParams};

/// Type macro to represent the inner hash type of [`FuzzyHash`]
/// (an instantiation of [`crate::internals::hash::FuzzyHash`]).
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
    ) -> Result<Self, crate::internals::errors::ParseError> {
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
    fn store_into_bytes(
        &self,
        out: &mut [u8],
    ) -> Result<usize, crate::internals::errors::OperationError> {
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
    #[inline(always)]
    fn clear_checksum(&mut self) {
        self.inner.clear_checksum()
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

/// The body part of the fuzzy hash.
pub mod body {
    pub use crate::internals::hash::body::{
        BODY_SIZE_LONG, BODY_SIZE_NORMAL, BODY_SIZE_SHORT, FuzzyHashBody, FuzzyHashBodyData,
    };
}

/// The checksum part of the fuzzy hash.
pub mod checksum {
    pub use crate::internals::hash::checksum::{
        CHECKSUM_SIZE_LONG, CHECKSUM_SIZE_NORMAL, FuzzyHashChecksum, FuzzyHashChecksumData,
    };
}

/// The Q ratio pair part of the fuzzy hash.
pub mod qratios {
    pub use crate::internals::hash::qratios::FuzzyHashQRatios;
}

mod tests;
