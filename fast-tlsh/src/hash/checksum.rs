// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The checksum part of the fuzzy hash.

use crate::buckets::constrained::{
    FuzzyHashBucketMapper, FuzzyHashBucketsInfo, LongFuzzyHashBucketMapper,
};
use crate::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};
use crate::compare::dist_checksum::{distance_1, distance_3};
use crate::errors::ParseError;
use crate::parse::hex_str::decode_rev_array;
use crate::pearson::tlsh_b_mapping_256;

/// The length of the normal (1-byte) checksum.
pub const CHECKSUM_SIZE_NORMAL: usize = 1;
/// The length of the long (3-byte) checksum.
pub const CHECKSUM_SIZE_LONG: usize = 3;

/// The private part.
pub(crate) mod private {
    /// The sealed trait.
    pub trait Sealed {}
}

/// The inner part.
pub(crate) mod inner {
    /// The trait representing "updating" behavior of the checksum.
    ///
    /// From the outside, only "read-only" part is public and "updating" part
    /// should be kept private in this crate.
    pub trait InnerChecksum: super::private::Sealed {
        /// Update the checksum by the last two bytes in the update window.
        fn update(&mut self, curr: u8, prev: u8);
    }

    /// The trait to provide one byte checksum validness checker.
    pub trait OneByteChecksumChecker: super::private::Sealed {
        /// Returns whether the given checksum value is valid.
        ///
        /// In the default implementation, it is always [`true`] because
        /// all values are valid.
        #[allow(unused_variables)]
        fn is_valid(checksum: u8) -> bool {
            true
        }
    }
}

/// Implementation provider for [`inner::OneByteChecksumChecker`].
struct OneByteChecksumChecker<const SIZE_BUCKETS: usize>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper;

impl private::Sealed for OneByteChecksumChecker<NUM_BUCKETS_SHORT> {}
impl inner::OneByteChecksumChecker for OneByteChecksumChecker<NUM_BUCKETS_SHORT> {
    /// Returns whether the given checksum value is valid.
    ///
    /// In the 48 bucket variant, it checks whether the value is
    /// **equal to or less than** [the number of buckets](NUM_BUCKETS_SHORT).
    fn is_valid(checksum: u8) -> bool {
        checksum <= NUM_BUCKETS_SHORT as u8
    }
}
impl private::Sealed for OneByteChecksumChecker<NUM_BUCKETS_NORMAL> {}
impl inner::OneByteChecksumChecker for OneByteChecksumChecker<NUM_BUCKETS_NORMAL> {}
impl private::Sealed for OneByteChecksumChecker<NUM_BUCKETS_LONG> {}
impl inner::OneByteChecksumChecker for OneByteChecksumChecker<NUM_BUCKETS_LONG> {}

/// The trait representing the checksum part of the fuzzy hash.
///
/// For the background of configurations, see [`FuzzyHashChecksumData`]
/// documentation.
pub trait FuzzyHashChecksum: inner::InnerChecksum {
    /// The size of the checksum.
    const SIZE: usize;
    /// The maximum distance between two checksums on this configuration.
    const MAX_DISTANCE: u32;
    /// Check whether the given checksum has a valid value.
    fn is_valid(&self) -> bool;
    /// Compare against another checksum and return the distance between them.
    fn compare(&self, other: &Self) -> u32;
}

/// The checksum part data of a fuzzy hash.
///
/// Note that, this is also parameterized by the number of buckets
/// (`SIZE_BUCKETS`) because, from the generator perspective,
/// it depends on the number of buckets.
///
/// This type supports following configurations:
///
/// *   1-byte checksum (on 48, 128, 256 bucket variants)
/// *   3-byte checksum (on 128, 256 bucket variants)
///
/// For the main functionalities except [`data()`](Self::data()),
/// see [`FuzzyHashChecksum`] documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FuzzyHashChecksumData<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
    /// The raw checksum.
    data: [u8; SIZE_CKSUM],
}

impl<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize>
    FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
    /// Initializes the checksum object with the initial state.
    pub(crate) fn new() -> Self {
        Self {
            data: [0; SIZE_CKSUM],
        }
    }

    /// Creates the checksum object from the raw data.
    pub(crate) fn from_raw(data: &[u8; SIZE_CKSUM]) -> Self {
        Self { data: *data }
    }

    /// Decode the object from a subset of
    /// the TLSH's hexadecimal representation.
    #[inline]
    pub(crate) fn from_str_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != SIZE_CKSUM * 2 {
            return Err(ParseError::InvalidStringLength);
        }
        let mut data = [0u8; SIZE_CKSUM];
        if decode_rev_array(&mut data, bytes) {
            Ok(Self { data })
        } else {
            Err(ParseError::InvalidCharacter)
        }
    }

    /// Returns the reference of raw checksum data.
    #[inline(always)]
    pub fn data(&self) -> &[u8; SIZE_CKSUM] {
        &self.data
    }
}

// Normal variant (1-byte checksum)
impl<const SIZE_BUCKETS: usize> private::Sealed
    for FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
}
impl<const SIZE_BUCKETS: usize> inner::InnerChecksum
    for FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
    #[inline(always)]
    fn update(&mut self, curr: u8, prev: u8) {
        self.data[0] = FuzzyHashBucketsInfo::<SIZE_BUCKETS>::b_mapping(0, curr, prev, self.data[0]);
    }
}
impl<const SIZE_BUCKETS: usize> FuzzyHashChecksum
    for FuzzyHashChecksumData<CHECKSUM_SIZE_NORMAL, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
    OneByteChecksumChecker<SIZE_BUCKETS>: inner::OneByteChecksumChecker,
{
    const SIZE: usize = CHECKSUM_SIZE_NORMAL;
    const MAX_DISTANCE: u32 = CHECKSUM_SIZE_NORMAL as u32;
    fn is_valid(&self) -> bool {
        use inner::OneByteChecksumChecker as _;
        OneByteChecksumChecker::<SIZE_BUCKETS>::is_valid(self.data[0])
    }
    #[inline(always)]
    fn compare(&self, other: &Self) -> u32 {
        distance_1(self.data, other.data)
    }
}

// Long variant (3-byte checksum)
impl<const SIZE_BUCKETS: usize> private::Sealed
    for FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: LongFuzzyHashBucketMapper,
{
}
impl<const SIZE_BUCKETS: usize> inner::InnerChecksum
    for FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: LongFuzzyHashBucketMapper,
{
    #[inline(always)]
    fn update(&mut self, curr: u8, prev: u8) {
        self.data[0] = FuzzyHashBucketsInfo::<SIZE_BUCKETS>::b_mapping(0, curr, prev, self.data[0]);
        self.data[1] = tlsh_b_mapping_256(self.data[0], curr, prev, self.data[1]);
        self.data[2] = tlsh_b_mapping_256(self.data[1], curr, prev, self.data[2]);
    }
}
impl<const SIZE_BUCKETS: usize> FuzzyHashChecksum
    for FuzzyHashChecksumData<CHECKSUM_SIZE_LONG, SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: LongFuzzyHashBucketMapper,
{
    const SIZE: usize = CHECKSUM_SIZE_LONG;
    const MAX_DISTANCE: u32 = CHECKSUM_SIZE_LONG as u32;
    #[inline(always)]
    fn is_valid(&self) -> bool {
        true
    }
    #[inline(always)]
    fn compare(&self, other: &Self) -> u32 {
        distance_3(self.data, other.data)
    }
}

mod tests;
