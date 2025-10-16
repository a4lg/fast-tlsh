// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The TLSH buckets and their mappings.

use crate::internals::generate::bucket_aggregation;
use crate::internals::hash::body::{BODY_SIZE_LONG, BODY_SIZE_NORMAL, BODY_SIZE_SHORT};
use crate::internals::pearson::{tlsh_b_mapping_48, tlsh_b_mapping_256};
use crate::internals::utils::Sealed;

/// The effective number of buckets on the short variant (with 48 buckets).
///
/// On this variant, we have at least 49 physical buckets but the last one is
/// used only to drain outliers.
///
/// In the official TLSH implementation, the variant with this number of
/// buckets is called "min hash".
pub const NUM_BUCKETS_SHORT: usize = 48;

/// The effective number of buckets on the normal variant.
///
/// On this variant, we have 256 physical buckets but only the first half
/// (128 buckets) are used to generate a fuzzy hash.
///
/// In the official TLSH implementation, the variant with this number of
/// buckets is called "compact hash".
pub const NUM_BUCKETS_NORMAL: usize = 128;

/// The effective number of buckets on the long variant (with 256 buckets).
///
/// In the official TLSH implementation, the variant with this number of
/// buckets is called "full hash".
pub const NUM_BUCKETS_LONG: usize = 256;

// Those sizes must be divisible by 4.
static_assertions::const_assert_eq!(NUM_BUCKETS_SHORT % 4, 0);
static_assertions::const_assert_eq!(NUM_BUCKETS_NORMAL % 4, 0);
static_assertions::const_assert_eq!(NUM_BUCKETS_LONG % 4, 0);

/// The trait to represent a bucket mapping.
pub trait FuzzyHashBucketMapper: Sealed {
    /// Raw bucket array type.
    type RawBucketType;
    /// Raw body type.
    type RawBodyType;
    /// Minimum non-zero buckets to be filled (inclusive).
    ///
    /// This is approximately half of the number of buckets but there's an
    /// exception of the short variant (slightly lower than the half).
    const MIN_NONZERO_BUCKETS: usize;
    /// TLSH's B (bucket) mapping suitable for corresponding implementation.
    fn b_mapping(b0: u8, b1: u8, b2: u8, b3: u8) -> u8;
    /// Denotes whether the B (bucket) mapping function is
    /// constrained to the bucket size.
    ///
    /// If this value is [`true`], all values returned by
    /// [`b_mapping()`](Self::b_mapping()) are less than the number of the
    /// buckets.  If not, some may be equal to or greater than that and will
    /// need to ignore such values by some means.
    const IS_B_MAPPING_CONSTRAINED_WITHIN_BUCKETS: bool;
    /// Bucket aggregation function.
    fn aggregate_buckets(
        out: &mut Self::RawBodyType,
        buckets: &Self::RawBucketType,
        q1: u32,
        q2: u32,
        q3: u32,
    );
}

/// A [`FuzzyHashBucketMapper`] implementation to switch implementation
/// by the number of buckets.
pub struct FuzzyHashBucketsInfo<const SIZE_BUCKETS: usize>;

// Short (48 bucket) bucket mapping implementation
impl Sealed for FuzzyHashBucketsInfo<NUM_BUCKETS_SHORT> {}
impl FuzzyHashBucketMapper for FuzzyHashBucketsInfo<NUM_BUCKETS_SHORT> {
    type RawBucketType = [u32; NUM_BUCKETS_SHORT];
    type RawBodyType = [u8; BODY_SIZE_SHORT];
    const MIN_NONZERO_BUCKETS: usize = 18;
    #[inline(always)]
    fn b_mapping(b0: u8, b1: u8, b2: u8, b3: u8) -> u8 {
        tlsh_b_mapping_48(b0, b1, b2, b3)
    }
    const IS_B_MAPPING_CONSTRAINED_WITHIN_BUCKETS: bool = false;
    #[inline(always)]
    fn aggregate_buckets(
        out: &mut Self::RawBodyType,
        buckets: &Self::RawBucketType,
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        bucket_aggregation::aggregate_48(out, buckets, q1, q2, q3);
    }
}

// Normal (128 bucket) bucket mapping implementation
impl Sealed for FuzzyHashBucketsInfo<NUM_BUCKETS_NORMAL> {}
impl FuzzyHashBucketMapper for FuzzyHashBucketsInfo<NUM_BUCKETS_NORMAL> {
    type RawBucketType = [u32; NUM_BUCKETS_NORMAL];
    type RawBodyType = [u8; BODY_SIZE_NORMAL];
    const MIN_NONZERO_BUCKETS: usize = NUM_BUCKETS_NORMAL / 2 + 1;
    #[inline(always)]
    fn b_mapping(b0: u8, b1: u8, b2: u8, b3: u8) -> u8 {
        // Note: use 256 bucket mapping (only first 128 for the hash body)
        tlsh_b_mapping_256(b0, b1, b2, b3)
    }
    const IS_B_MAPPING_CONSTRAINED_WITHIN_BUCKETS: bool = false;
    #[inline(always)]
    fn aggregate_buckets(
        out: &mut Self::RawBodyType,
        buckets: &Self::RawBucketType,
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        bucket_aggregation::aggregate_128(out, buckets, q1, q2, q3);
    }
}

// Long (256 bucket) bucket mapping implementation
impl Sealed for FuzzyHashBucketsInfo<NUM_BUCKETS_LONG> {}
impl FuzzyHashBucketMapper for FuzzyHashBucketsInfo<NUM_BUCKETS_LONG> {
    type RawBucketType = [u32; NUM_BUCKETS_LONG];
    type RawBodyType = [u8; BODY_SIZE_LONG];
    const MIN_NONZERO_BUCKETS: usize = NUM_BUCKETS_LONG / 2 + 1;
    #[inline(always)]
    fn b_mapping(b0: u8, b1: u8, b2: u8, b3: u8) -> u8 {
        tlsh_b_mapping_256(b0, b1, b2, b3)
    }
    const IS_B_MAPPING_CONSTRAINED_WITHIN_BUCKETS: bool = true;
    #[inline(always)]
    fn aggregate_buckets(
        out: &mut Self::RawBodyType,
        buckets: &Self::RawBucketType,
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        bucket_aggregation::aggregate_256(out, buckets, q1, q2, q3);
    }
}

/// The trait representing a "longer" bucket mapping implementation.
///
/// On 48 bucket (short) variants of the fuzzy hash, the checksum value
/// cannot have the length of 3 (long).
///
/// This trait is implemented by bucket mappers with 3-byte checksum
/// is possible to define.
pub trait LongFuzzyHashBucketMapper: FuzzyHashBucketMapper {}
impl LongFuzzyHashBucketMapper for FuzzyHashBucketsInfo<NUM_BUCKETS_NORMAL> {}
impl LongFuzzyHashBucketMapper for FuzzyHashBucketsInfo<NUM_BUCKETS_LONG> {}

/// TLSH bucket data.
///
/// By default, it consists of 256-entry of [`u32`] buckets to reduce branches.
///
/// If the feature `opt-low-memory-buckets` is enabled, the number of entries
/// will be reduced to the actual effective bucket size (`SIZE_BUCKETS`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(align(16))]
pub(crate) struct FuzzyHashBucketsData<const SIZE_BUCKETS: usize>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
    /// The contents of the buckets.
    #[cfg(not(feature = "opt-low-memory-buckets"))]
    pub(crate) buckets: [u32; 256],
    /// The contents of the buckets.
    #[cfg(feature = "opt-low-memory-buckets")]
    pub(crate) buckets: [u32; SIZE_BUCKETS],
}
impl<const SIZE_BUCKETS: usize> FuzzyHashBucketsData<SIZE_BUCKETS>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
{
    /// Creates the new buckets object.
    pub(crate) fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "opt-low-memory-buckets"))] {
                Self { buckets: [0; 256] }
            } else {
                Self { buckets: [0; SIZE_BUCKETS] }
            }
        }
    }

    /// Returns the reference to the data (as a slice).
    #[inline(always)]
    pub(crate) fn data(&self) -> &[u32] {
        &self.buckets[..SIZE_BUCKETS]
    }

    /// Increment a bucket specified by the index.
    ///
    /// By default, it increments the specified bucket no matter what.
    /// Because the index is in [`u8`] and we have 256-entry buckets,
    /// it will not cause any buffer overflow (memory-safe).
    ///
    /// If you turn on the feature `opt-low-memory-buckets`, it ignores
    /// the index which does not fit in the internal bucket array.
    /// It will make the program slightly slower but also is memory-safe.
    #[inline(always)]
    pub(crate) fn increment(&mut self, index: u8) {
        let index = index as usize;
        #[cfg(feature = "opt-low-memory-buckets")]
        if !FuzzyHashBucketsInfo::<SIZE_BUCKETS>::IS_B_MAPPING_CONSTRAINED_WITHIN_BUCKETS
            && index >= SIZE_BUCKETS
        {
            return;
        }
        self.buckets[index] = self.buckets[index].wrapping_add(1);
    }
}
