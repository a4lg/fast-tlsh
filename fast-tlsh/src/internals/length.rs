// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Data length encodings and other handlings.

use core::ops::RangeInclusive;

use crate::internals::buckets::{
    FuzzyHashBucketMapper, FuzzyHashBucketsInfo, NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL,
    NUM_BUCKETS_SHORT,
};
use crate::internals::compare::dist_length::{distance, MAX_DISTANCE};
use crate::internals::errors::ParseError;
#[allow(unused_imports)]
use crate::internals::macros::{invariant, optionally_unsafe};
use crate::internals::parse::hex_str::decode_rev_1;
use crate::internals::utils::Sealed;

/// The number of valid encoded length values.
///
/// Because the length is encoded as an 8-bit integer, this constant shall be
/// equal to or less than 2<sup>8</sup> (`256`).
pub const ENCODED_VALUE_SIZE: usize = 170;
static_assertions::const_assert!(ENCODED_VALUE_SIZE <= 256);

/// Top values for each data length encodings.
///
/// This array is strictly increasing and encodes the *maximum* data length
/// (inclusive) for each 8-bit encoded value.
#[rustfmt::skip]
const TOP_VALUE_BY_ENCODING: [u32; ENCODED_VALUE_SIZE] = [
    // 0x00-0x0f: == floor(1.5^(i+1))
    1,
    2,
    3,
    5,
    7,
    11,
    17,
    25,
    38,
    57,
    86,
    129,
    194,
    291,
    437,
    656,
    // 0x10-0x15: == floor(657 * 1.3^(i-0x10+1))
    854,
    1110,
    1443,
    1876,
    2439,
    3171,
    // 0x16-0xa9 : ~= floor(3159.625 * 1.1^(i-0x16+1))
    // [gets inexact as the index increases]
    3475,
    3823,
    4205,
    4626,
    5088,
    5597,
    6157,
    6772,
    7450,
    8195,
    9014,
    9916,
    10907,
    11998,
    13198,
    14518,
    15970,
    17567,
    19323,
    21256,
    23382,
    25720,
    28292,
    31121,
    34233,
    37656,
    41422,
    45564,
    50121,
    55133,
    60646,
    66711,
    73382,
    80721,
    88793,
    97672,
    107439,
    118183,
    130002,
    143002,
    157302,
    173032,
    190335,
    209369,
    230306,
    253337,
    278670,
    306538,
    337191,
    370911,
    408002,
    448802,
    493682,
    543050,
    597356,
    657091,
    722800,
    795081,
    874589,
    962048,
    1058252,
    1164078,
    1280486,
    1408534,
    1549388,
    1704327,
    1874759,
    2062236,
    2268459,
    2495305,
    2744836,
    3019320,
    3321252,
    3653374,
    4018711,
    4420582,
    4862641,
    5348905,
    5883796,
    6472176,
    7119394,
    7831333,
    8614467,
    9475909,
    10423501,
    11465851,
    12612437,
    13873681,
    15261050,
    16787154,
    18465870,
    20312458,
    22343706,
    24578077,
    27035886,
    29739474,
    32713425,
    35984770,
    39583245,
    43541573,
    47895730,
    52685306,
    57953837,
    63749221,
    70124148,
    77136564,
    84850228,
    93335252,
    102668779,
    112935659,
    124229227,
    136652151,
    150317384,
    165349128,
    181884040,
    200072456,
    220079703,
    242087671,
    266296456,
    292926096,
    322218735,
    354440623,
    389884688,
    428873168,
    471760495,
    518936559,
    570830240,
    627913311,
    690704607,
    759775136,
    835752671,
    919327967,
    1011260767,
    1112386880,
    1223623232,
    1345985727,
    1480584256,
    1628642751,
    1791507135,
    1970657856,
    2167723648,
    2384496256,
    2622945920,
    2885240448,
    3173764736,
    3491141248,
    3840255616,
    4224281216,
];

/// The maximum data length (inclusive).
pub(crate) const MAX: u32 = TOP_VALUE_BY_ENCODING[TOP_VALUE_BY_ENCODING.len() - 1];

/// Denotes bucket count-specific length constraints.
pub trait ConstrainedLengthProcessingInfo: Sealed {
    /// The minimum data length (on [all modes](DataLengthProcessingMode)).
    const MIN: u32;
    /// The minimum data length (on [the conservative mode](DataLengthProcessingMode::Conservative)).
    const MIN_CONSERVATIVE: u32;
    /// The maximum data length (inclusive).
    ///
    /// Note that this value is the same across all
    /// [length processing information](LengthProcessingInfo) instantiations.
    const MAX: u32 = self::MAX;
}

/// Length processing information (depending on the number of buckets).
///
/// A valid instantiation of this type (constrained by a private trait)
/// implements the sealed trait [`ConstrainedLengthProcessingInfo`].
pub struct LengthProcessingInfo<const SIZE_BUCKETS: usize>
where
    FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper;
// Short (48 bucket) information
impl Sealed for LengthProcessingInfo<NUM_BUCKETS_SHORT> where
    FuzzyHashBucketsInfo<NUM_BUCKETS_SHORT>: FuzzyHashBucketMapper
{
}
impl ConstrainedLengthProcessingInfo for LengthProcessingInfo<NUM_BUCKETS_SHORT>
where
    FuzzyHashBucketsInfo<NUM_BUCKETS_SHORT>: FuzzyHashBucketMapper,
{
    const MIN: u32 = 10;
    const MIN_CONSERVATIVE: u32 = 10;
}
// Normal (128 bucket) information
impl Sealed for LengthProcessingInfo<NUM_BUCKETS_NORMAL> where
    FuzzyHashBucketsInfo<NUM_BUCKETS_NORMAL>: FuzzyHashBucketMapper
{
}
impl ConstrainedLengthProcessingInfo for LengthProcessingInfo<NUM_BUCKETS_NORMAL>
where
    FuzzyHashBucketsInfo<NUM_BUCKETS_NORMAL>: FuzzyHashBucketMapper,
{
    const MIN: u32 = 50;
    const MIN_CONSERVATIVE: u32 = 128;
}
// Long (256 bucket) information
impl Sealed for LengthProcessingInfo<NUM_BUCKETS_LONG> where
    FuzzyHashBucketsInfo<NUM_BUCKETS_LONG>: FuzzyHashBucketMapper
{
}
impl ConstrainedLengthProcessingInfo for LengthProcessingInfo<NUM_BUCKETS_LONG>
where
    FuzzyHashBucketsInfo<NUM_BUCKETS_LONG>: FuzzyHashBucketMapper,
{
    const MIN: u32 = 50;
    const MIN_CONSERVATIVE: u32 = 128;
}

/// The first index of [`TOP_VALUE_BY_ENCODING`] which *exceeds*
/// 2<sup>n</sup> with the specified count of leading zeros.
///
/// It can be used to narrow the search space of [`TOP_VALUE_BY_ENCODING`]
/// by using the index `clz` as the top and the index `clz+1` as the bottom,
/// and using [`TOP_VALUE_BY_ENCODING`]`[bottom..top]` as the search space.
#[cfg(any(
    test,
    doc,
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
        all(
            any(target_arch = "riscv32", target_arch = "riscv64"),
            target_feature = "zbb"
        ),
        target_arch = "wasm32",
        target_arch = "wasm64"
    )
))]
const ENCODED_INDICES_BY_LEADING_ZEROS: [usize; 33] = {
    let mut array = [0; 33];
    let mut i = 0;
    while i < TOP_VALUE_BY_ENCODING.len() {
        array[TOP_VALUE_BY_ENCODING[i].leading_zeros() as usize] = i + 1;
        i += 1;
    }
    array
};

/// Denotes validity depending on the data length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLengthValidity {
    /// Too small to process (even in the default optimistic mode).
    TooSmall,
    /// Valid on the optimistic mode (default) but invalid
    /// (too small) on the conservative mode.
    ValidWhenOptimistic,
    /// Valid *also* on the conservative mode (i.e. valid on all modes).
    Valid,
    /// Too large to process.
    TooLarge,
}

/// Denotes processing mode depending on the input data length.
///
/// This type can be specified in following methods:
///
/// *   [`DataLengthValidity::is_err_on()`]
/// *   [`GeneratorOptions::length_processing_mode()`](crate::GeneratorOptions::length_processing_mode())
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataLengthProcessingMode {
    /// The optimistic mode (the default on the official implementation).
    ///
    /// It allows processing small files knowing that some of them are not very
    /// useful for fuzzy comparison.
    #[default]
    Optimistic,
    /// The conservative mode.
    ///
    /// It was the default mode in the past official implementation.  While not
    /// always true, it generates statistically better fuzzy hashes if
    /// the generator accepts.
    Conservative,
}

impl DataLengthValidity {
    /// Gets the validity value depending on the input data length and
    /// the number of buckets.
    ///
    /// The `SIZE_BUCKETS` parameter shall be the one of the bucket size
    /// constants in [`tlsh::buckets`](crate::buckets).
    pub fn new<const SIZE_BUCKETS: usize>(len: u32) -> DataLengthValidity
    where
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        if len < LengthProcessingInfo::<SIZE_BUCKETS>::MIN {
            DataLengthValidity::TooSmall
        } else if len < LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE {
            DataLengthValidity::ValidWhenOptimistic
        } else if len <= LengthProcessingInfo::<SIZE_BUCKETS>::MAX {
            DataLengthValidity::Valid
        } else {
            DataLengthValidity::TooLarge
        }
    }

    /// Checks whether this validity value is a hard error
    /// (i.e. whether this is an error on all modes).
    pub fn is_err(&self) -> bool {
        matches!(
            *self,
            DataLengthValidity::TooSmall | DataLengthValidity::TooLarge
        )
    }

    /// Checks whether this validity value is an error
    /// on the specified processing mode.
    pub fn is_err_on(&self, mode: DataLengthProcessingMode) -> bool {
        match *self {
            DataLengthValidity::TooLarge | DataLengthValidity::TooSmall => true,
            DataLengthValidity::Valid => false,
            DataLengthValidity::ValidWhenOptimistic => {
                matches!(mode, DataLengthProcessingMode::Conservative)
            }
        }
    }
}

/// Approximated input data length encoded as 8-bits in a fuzzy hash.
///
/// On TLSH, it compresses a 32-bit input size to an approximated encoding of
/// 8-bits.  This enables to distinguish statistically similar files with
/// large differences in the size.
///
/// This struct can have a:
///
/// 1.  A valid encoding for a valid input size,
/// 2.  A "valid" encoding for an invalid input size
///     (only appears when the input is smaller than the TLSH's lower limit), or
/// 3.  An invalid encoding (does not correspond any of the 32-bit size).
///
/// This struct only handles the validness of its encoding.
/// So, the case 2 above is considered "valid" in this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FuzzyHashLengthEncoding {
    /// The raw (approximated) length encoding.
    lvalue: u8,
}
impl FuzzyHashLengthEncoding {
    /// The maximum distance between two length encodings.
    pub const MAX_DISTANCE: u32 = MAX_DISTANCE;

    /// Creates the object from the raw encoding.
    #[inline(always)]
    pub(crate) fn from_raw(lvalue: u8) -> Self {
        Self { lvalue }
    }

    /// Decode the object from a subset of
    /// the TLSH's hexadecimal representation.
    pub(crate) fn from_str_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes.len() != 2 {
            return Err(ParseError::InvalidStringLength);
        }
        decode_rev_1(bytes)
            .ok_or(ParseError::InvalidCharacter)
            .map(Self::from_raw)
    }

    /// Encode the 32-bit data length as rough 8-bit representation.
    pub fn new(len: u32) -> Option<Self> {
        if len == 0 {
            return Some(Self { lvalue: 0 }); // hard error (too small) but for consistency
        }
        if len > MAX {
            return None;
        }
        cfg_if::cfg_if! {
            // Note: "arm" assumes ARMv7+ (CLZ is first implemented in ARMv5T).
            // Note: On WASM, whether "i32.clz" is efficient depends on the
            //       implementation but it should be safe enough to assume that
            //       considering major CPUs used to run WebAssembly.
            if #[cfg(any(
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "arm",
                target_arch = "aarch64",
                all(
                    any(target_arch = "riscv32", target_arch = "riscv64"),
                    target_feature = "zbb"
                ),
                target_arch = "wasm32",
                target_arch = "wasm64"
            ))] {
                let clz = len.leading_zeros() as usize;
                let bottom = ENCODED_INDICES_BY_LEADING_ZEROS[clz + 1];
                let top = ENCODED_INDICES_BY_LEADING_ZEROS[clz];
                optionally_unsafe! {
                    invariant!(bottom <= TOP_VALUE_BY_ENCODING.len());
                    invariant!(top <= TOP_VALUE_BY_ENCODING.len());
                    invariant!(bottom <= top);
                }
                Some(Self {
                    lvalue: match TOP_VALUE_BY_ENCODING[bottom..top].binary_search(&len) {
                        Ok(i) => bottom + i,
                        Err(i) => bottom + i,
                    } as u8,
                })
            }
            else {
                Some(Self {
                    lvalue: match TOP_VALUE_BY_ENCODING.as_slice().binary_search(&len) {
                        Ok(i) => i,
                        Err(i) => i,
                    } as u8,
                })
            }
        }
    }

    /// Returns the raw encoding.
    #[inline(always)]
    pub fn value(&self) -> u8 {
        self.lvalue
    }

    /// Returns whether the encoding is valid.
    ///
    /// Note that, if the encoding only appears when the input size is too
    /// small, it also returns [`true`] because the encoding itself is still
    /// valid.  On the other hand, if the encoding exceeds the upper limit, it
    /// will return [`false`] because it will not correspond to any of 32-bit
    /// input size.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        (self.lvalue as usize) < ENCODED_VALUE_SIZE
    }

    /// Compare against another length encoding and
    /// return the distance between them.
    #[inline(always)]
    pub fn compare(&self, other: &FuzzyHashLengthEncoding) -> u32 {
        distance(self.lvalue, other.lvalue)
    }

    /// Decode the encoded 8-bit length approximation as an inclusive 32-bit
    /// input size range that will produce the given encoding.
    ///
    /// It will return [`None`] if there's no valid 32-bit size for given
    /// encoding.
    pub fn range(&self) -> Option<RangeInclusive<u32>> {
        if self.lvalue == 0 {
            return Some(0..=TOP_VALUE_BY_ENCODING[0]);
        }
        if self.lvalue as usize >= ENCODED_VALUE_SIZE {
            return None;
        }
        let bottom = TOP_VALUE_BY_ENCODING[self.lvalue as usize - 1] + 1;
        let top = TOP_VALUE_BY_ENCODING[self.lvalue as usize];
        Some(bottom..=top)
    }
}
impl TryFrom<u32> for FuzzyHashLengthEncoding {
    type Error = ParseError;

    fn try_from(len: u32) -> Result<Self, Self::Error> {
        Self::new(len).ok_or(ParseError::LengthIsTooLarge)
    }
}

/// Encode the 32-bit data length as rough 8-bit representation.
#[cfg(any(doc, test))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
fn encode(len: u32) -> Option<u8> {
    FuzzyHashLengthEncoding::new(len).map(|x| x.lvalue)
}

/// The naïve implementation.
#[cfg(any(doc, test))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
pub(crate) mod naive {
    use super::TOP_VALUE_BY_ENCODING;

    /// Encode the 32-bit data length as rough 8-bit representation.
    pub fn encode(len: u32) -> Option<u8> {
        if len == 0 {
            return Some(0); // hard error (too small) but for consistency
        }
        for (i, &top) in TOP_VALUE_BY_ENCODING.iter().enumerate() {
            if len <= top {
                return Some(i as u8);
            }
        }
        None
    }
}

mod tests;
