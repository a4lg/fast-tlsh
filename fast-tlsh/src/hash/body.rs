// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The body part of the fuzzy hash.

use crate::compare::dist_body::{
    distance_12, distance_32, distance_64, MAX_DISTANCE_LONG, MAX_DISTANCE_NORMAL,
    MAX_DISTANCE_SHORT,
};
use crate::errors::ParseError;
use crate::internals::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};

#[cfg(not(feature = "opt-simd-parse-hex"))]
use crate::internals::parse::hex_str::decode_array;

/// The body size of the short variant (with 48 effective buckets).
///
/// Because we need 2-bits body for each bucket, this is the quarter of
/// [the number of effective buckets](crate::buckets::NUM_BUCKETS_SHORT).
pub const BODY_SIZE_SHORT: usize = NUM_BUCKETS_SHORT / 4;

/// The body size of the normal variant (with 128 effective buckets).
///
/// Because we need 2-bits body for each bucket, this is the quarter of
/// [the number of effective buckets](crate::buckets::NUM_BUCKETS_NORMAL).
pub const BODY_SIZE_NORMAL: usize = NUM_BUCKETS_NORMAL / 4;

/// The body size of the long variant (with 256 effective buckets).
///
/// Because we need 2-bits body for each bucket, this is the quarter of
/// [the number of effective buckets](crate::buckets::NUM_BUCKETS_LONG).
pub const BODY_SIZE_LONG: usize = NUM_BUCKETS_LONG / 4;

/// The private part.
mod private {
    /// The sealed trait.
    pub trait Sealed {}
}

/// The trait representing the body part of the fuzzy hash.
pub trait FuzzyHashBody: private::Sealed {
    /// The number of buckets in the body.
    const NUM_BUCKETS: usize;
    /// The size of the body in bytes.
    const SIZE: usize;
    /// The maximum distance between two bodies on this configuration.
    const MAX_DISTANCE: u32;
    /// Retrieves the quartile value (`0b00..=0b11`) for specified bucket.
    ///
    /// # Safety
    ///
    /// The `index` argument must be less than [`NUM_BUCKETS`](Self::NUM_BUCKETS)
    /// or otherwise results in a panic.
    fn quartile(&self, index: usize) -> u8;
    /// Compare against another body and return the distance between them.
    fn compare(&self, other: &Self) -> u32;
}

/// The body part data of the fuzzy hash.
///
/// For the main functionalities, see [`FuzzyHashBody`] documentation.
#[repr(align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyHashBodyData<const SIZE_BODY: usize> {
    /// The raw body data.
    data: [u8; SIZE_BODY],
}

impl<const SIZE_BODY: usize> FuzzyHashBodyData<SIZE_BODY> {
    /// Creates an object from the existing body.
    pub(crate) fn from_raw(data: [u8; SIZE_BODY]) -> Self {
        Self { data }
    }

    /// Decode the object from a subset of
    /// the TLSH's hexadecimal representation.
    #[inline]
    pub(crate) fn from_str_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != SIZE_BODY * 2 {
            return Err(ParseError::InvalidStringLength);
        }
        let mut data = [0u8; SIZE_BODY];
        cfg_if::cfg_if! {
            if #[cfg(feature = "opt-simd-parse-hex")] {
                let result =
                    hex_simd::decode(bytes, hex_simd::Out::from_slice(data.as_mut_slice())).is_ok();
            } else {
                let result = decode_array(&mut data, bytes);
            }
        }
        if result {
            Ok(Self { data })
        } else {
            Err(ParseError::InvalidCharacter)
        }
    }

    /// Returns the raw data corresponding the body part.
    #[inline(always)]
    pub fn data(&self) -> &[u8; SIZE_BODY] {
        &self.data
    }
}

// Short (48 bucket) body implementation
impl private::Sealed for FuzzyHashBodyData<BODY_SIZE_SHORT> {}
impl FuzzyHashBody for FuzzyHashBodyData<BODY_SIZE_SHORT> {
    const NUM_BUCKETS: usize = NUM_BUCKETS_SHORT;
    const SIZE: usize = BODY_SIZE_SHORT;
    const MAX_DISTANCE: u32 = MAX_DISTANCE_SHORT;
    #[inline(always)]
    fn quartile(&self, index: usize) -> u8 {
        assert!(index < Self::NUM_BUCKETS);
        (self.data[self.data.len() - 1 - index / 4] >> (2 * (index % 4))) & 0b11
    }
    #[inline(always)]
    fn compare(&self, other: &Self) -> u32 {
        distance_12(&self.data, &other.data)
    }
}

// Normal (128 bucket) body implementation
impl private::Sealed for FuzzyHashBodyData<BODY_SIZE_NORMAL> {}
impl FuzzyHashBody for FuzzyHashBodyData<BODY_SIZE_NORMAL> {
    const NUM_BUCKETS: usize = NUM_BUCKETS_NORMAL;
    const SIZE: usize = BODY_SIZE_NORMAL;
    const MAX_DISTANCE: u32 = MAX_DISTANCE_NORMAL;
    #[inline(always)]
    fn quartile(&self, index: usize) -> u8 {
        assert!(index < Self::NUM_BUCKETS);
        (self.data[self.data.len() - 1 - index / 4] >> (2 * (index % 4))) & 0b11
    }
    #[inline(always)]
    fn compare(&self, other: &Self) -> u32 {
        distance_32(&self.data, &other.data)
    }
}

// Long (256 bucket) body implementation
impl private::Sealed for FuzzyHashBodyData<BODY_SIZE_LONG> {}
impl FuzzyHashBody for FuzzyHashBodyData<BODY_SIZE_LONG> {
    const NUM_BUCKETS: usize = NUM_BUCKETS_LONG;
    const SIZE: usize = BODY_SIZE_LONG;
    const MAX_DISTANCE: u32 = MAX_DISTANCE_LONG;
    #[inline(always)]
    fn quartile(&self, index: usize) -> u8 {
        assert!(index < Self::NUM_BUCKETS);
        (self.data[self.data.len() - 1 - index / 4] >> (2 * (index % 4))) & 0b11
    }
    #[inline(always)]
    fn compare(&self, other: &Self) -> u32 {
        distance_64(&self.data, &other.data)
    }
}

mod tests;
