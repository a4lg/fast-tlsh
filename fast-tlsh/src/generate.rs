// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The fuzzy hash generator.

use crate::errors::GeneratorError;
use crate::hash::body::{FuzzyHashBody, FuzzyHashBodyData};
use crate::hash::checksum::inner::InnerChecksum;
use crate::hash::checksum::{FuzzyHashChecksum, FuzzyHashChecksumData};
use crate::hash::qratios::FuzzyHashQRatios;
use crate::internals::buckets::{
    FuzzyHashBucketMapper, FuzzyHashBucketsData, FuzzyHashBucketsInfo,
};
use crate::internals::intrinsics::{likely, unlikely};
use crate::internals::macros::{invariant, optionally_unsafe};
use crate::internals::params::{
    ConstrainedFuzzyHashParams, ConstrainedFuzzyHashType, ConstrainedVerboseFuzzyHashParams,
    VerboseFuzzyHashParams,
};
use crate::length::{
    ConstrainedLengthProcessingInfo, DataLengthProcessingMode, DataLengthValidity,
    FuzzyHashLengthEncoding, LengthProcessingInfo,
};
use crate::{FuzzyHashType, GeneratorType};

pub(crate) mod bucket_aggregation;

/// Window size to obtain local features.
///
/// In the TLSH generator, we use a sliding window over the input to
/// capture local features.  In other words, to obtain local feature
/// information, only data inside the window is used.  This way, we'll get the
/// same hash local feature value even if some segments are moved.
///
/// This constant is not designed to be easily configurable.  In the original
/// implementation, it was configurable between 4–8 but we rarely use a
/// non-default constant.
pub const WINDOW_SIZE: usize = 5;

bitflags::bitflags! {
    /// TLSH-compatible generator option flags.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TLSHCompatibleGeneratorFlags: u8 {
        /// If set, the generator computes Q ratio values using only
        /// integers (unlike f32 as in the original implementation).
        const PURE_INTEGER_QRATIO_COMPUTATION = 0x01;
    }

    /// TLSH-incompatible generator option flags.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TLSHIncompatibleGeneratorFlags: u8 {
        /// If set, it allows smaller file sizes (even smaller than 50 bytes).
        ///
        /// But the will likely statistically weak.  You may need to enable
        /// [`ALLOW_STATISTICALLY_WEAK_BUCKETS_HALF`](Self::ALLOW_STATISTICALLY_WEAK_BUCKETS_HALF) and
        /// [`ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER`](Self::ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER).
        const ALLOW_SMALL_SIZE_FILES                   = 0x01;
        /// If set, it allows statistically weak buckets
        /// (approximately half or more are empty).
        const ALLOW_STATISTICALLY_WEAK_BUCKETS_HALF    = 0x02;
        /// If set, it allows statistically weak buckets
        /// (approximately 3/4 or more are empty).
        const ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER = 0x04;
    }
}

/// The object to group all generator options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorOptions {
    /// Current processing mode of the data length.
    length_mode: DataLengthProcessingMode,
    /// Flags indicating TLSH-compatible flags.
    compat_flags: TLSHCompatibleGeneratorFlags,
    /// Flags indicating TLSH-incompatible flags.
    incompat_flags: TLSHIncompatibleGeneratorFlags,
}

impl GeneratorOptions {
    /// Creates the default generator options.
    pub fn new() -> Self {
        Self {
            length_mode: Default::default(),
            compat_flags: TLSHCompatibleGeneratorFlags::empty(),
            incompat_flags: TLSHIncompatibleGeneratorFlags::empty(),
        }
    }

    /// Query whether this generator options are compatible to the official
    /// implementation of TLSH.
    ///
    /// If any of the options that are incompatible with the official TLSH
    /// implementation is set, this method will return [`false`].
    ///
    /// Otherwise, it returns [`true`].
    ///
    /// # Example
    ///
    /// ```
    /// use tlsh::generate::GeneratorOptions;
    ///
    /// let mut options = GeneratorOptions::new();
    /// // By default, the option is compatible to the official implementation.
    /// assert!(options.is_tlsh_compatible());
    /// // By allowing statistically weak hashes, it becomes incompatible with
    /// // the official implementation.
    /// let options = options.allow_small_size_files(true);
    /// assert!(!options.is_tlsh_compatible());
    /// ```
    pub fn is_tlsh_compatible(&self) -> bool {
        self.incompat_flags.is_empty()
    }

    /// Set the data length processing mode.
    ///
    /// For more information, see [`DataLengthProcessingMode`].
    ///
    /// # Example
    ///
    /// ```
    /// use core::str::FromStr;
    /// use tlsh::prelude::*;
    /// use tlsh::{GeneratorErrorCategory, GeneratorOptions};
    /// use tlsh::length::DataLengthProcessingMode;
    ///
    /// let mut generator = TlshGenerator::new();
    ///
    /// // With default options, relatively small data (50 bytes) is accepted.
    /// generator.update(b"Lovak won the squad prize cup for sixty big jumps.");
    /// let hash = generator.finalize().unwrap();
    /// let expected = "T14A90024954691E114404124180D942C1450F8423775ADE1510211420456593621A8173";
    /// let expected = Tlsh::from_str(expected).unwrap();
    /// assert_eq!(hash, expected);
    ///
    /// // But with conservative mode, it fails.
    /// // The failure is caused by an "invalid" length (in the conservatide mode).
    /// let result = generator.finalize_with_options(
    ///     GeneratorOptions::new()
    ///         .length_processing_mode(DataLengthProcessingMode::Conservative)
    /// );
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.category(), GeneratorErrorCategory::DataLength);
    /// ```
    pub fn length_processing_mode(&mut self, value: DataLengthProcessingMode) -> &mut Self {
        self.length_mode = value;
        self
    }

    /// Set whether we compute Q ratio values by pure integers.
    ///
    /// The official implementation (up to version 4.12.0) effectively uses
    /// [`f32`] for computing Q ratio values.  Enabling this option will make
    /// this computation purely integer-based (involving [`u64`]).
    ///
    /// This is [`true`] by default.
    ///
    /// # Compatibility
    ///
    /// The Q ratio computation algorithm is equivalent to following versions:
    ///
    /// *   [`true`] (default): TLSH 4.12.1+
    /// *   [`false`]: TLSH -4.12.0
    pub fn pure_integer_qratio_computation(&mut self, value: bool) -> &mut Self {
        self.compat_flags.set(
            TLSHCompatibleGeneratorFlags::PURE_INTEGER_QRATIO_COMPUTATION,
            value,
        );
        self
    }

    /// (fast-tlsh specific)
    /// Set whether we allow generating fuzzy hashes from very small inputs.
    ///
    /// **Warning**: This is a TLSH-incompatible option.
    ///
    /// # Example
    ///
    /// ```
    /// use core::str::FromStr;
    /// use tlsh::prelude::*;
    /// use tlsh::{GeneratorErrorCategory, GeneratorOptions};
    ///
    /// let mut generator = TlshGenerator::new();
    ///
    /// // With default options, very small data (44 bytes) is rejected
    /// // because it's smaller than the lower limit, 50 bytes.
    /// // The failure is caused by an "invalid" length.
    /// generator.update(b"The quick brown fox jumps over the lazy dog.");
    /// let result = generator.finalize();
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.category(), GeneratorErrorCategory::DataLength);
    ///
    /// // But with extended permissive mode, it succeeds
    /// // (it's also because the input is not statistically bad for TLSH).
    /// let hash = generator.finalize_with_options(
    ///     GeneratorOptions::new().allow_small_size_files(true)
    /// ).unwrap();
    /// let expected = "T19E90024A21181294648A1888438D94B292C8C510612114116430600218082219C98551";
    /// let expected = Tlsh::from_str(expected).unwrap();
    /// assert_eq!(hash, expected);
    /// ```
    pub fn allow_small_size_files(&mut self, value: bool) -> &mut Self {
        self.incompat_flags.set(
            TLSHIncompatibleGeneratorFlags::ALLOW_SMALL_SIZE_FILES,
            value,
        );
        self
    }

    /// (fast-tlsh specific)
    /// Set whether we allow generating fuzzy hashes from
    /// statistically weak buckets
    /// (when approximately half or more of them are empty).
    ///
    /// **Warning**: This is a TLSH-incompatible option.
    ///
    /// Note that this is a subset of
    /// [`allow_statistically_weak_buckets_quarter()`](Self::allow_statistically_weak_buckets_quarter()).
    /// If you set [`true`] using that method, this parameter is also
    /// considered [`true`] (regardless of the actual value inside).
    ///
    /// # Example
    ///
    /// ```
    /// use core::str::FromStr;
    /// use tlsh::prelude::*;
    /// use tlsh::{GeneratorErrorCategory, GeneratorOptions};
    ///
    /// let mut generator = TlshGenerator::new();
    ///
    /// // With default options, this data (50 bytes) generates statistically
    /// // weak hash (and thus rejected by default).
    /// // The failure is caused by an unbalanced data distribution.
    /// generator.update(b"ABCDEFGHIJKLMNOPQRSTABCDEFGHIJKLMNOPQRSTABCDEFGHIJ");
    /// let result = generator.finalize();
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.category(), GeneratorErrorCategory::DataDistribution);
    ///
    /// // But with extended permissive mode, it succeeds
    /// // (but you can see that there are too many zeroes which will make
    /// //  the comparison less useful).
    /// let hash = generator.finalize_with_options(
    ///     GeneratorOptions::new().allow_statistically_weak_buckets_half(true)
    /// ).unwrap();
    /// let expected = "T1609000080C838F2A0F2C82C0ECA282F33808838B00CE0300228C2F80C8800E08800000";
    /// let expected = Tlsh::from_str(expected).unwrap();
    /// assert_eq!(hash.to_string(), expected.to_string());
    /// ```
    pub fn allow_statistically_weak_buckets_half(&mut self, value: bool) -> &mut Self {
        self.incompat_flags.set(
            TLSHIncompatibleGeneratorFlags::ALLOW_STATISTICALLY_WEAK_BUCKETS_HALF,
            value,
        );
        self
    }

    /// (fast-tlsh specific)
    /// Set whether we allow generating fuzzy hashes from
    /// statistically weak buckets
    /// (when approximately 3/4 or more of them are empty).
    ///
    /// **Warning**: This is a TLSH-incompatible option.
    ///
    /// Note that this is a superset of
    /// [`allow_statistically_weak_buckets_half()`](Self::allow_statistically_weak_buckets_half()).
    /// If you set [`true`] using this method, it will ignore the parameter set by
    /// [`allow_statistically_weak_buckets_half()`](Self::allow_statistically_weak_buckets_half()).
    ///
    /// # Example
    ///
    /// ```
    /// use core::str::FromStr;
    /// use tlsh::prelude::*;
    /// use tlsh::{GeneratorErrorCategory, GeneratorOptions};
    ///
    /// let mut generator = TlshGenerator::new();
    ///
    /// // With default options or only half-bucket empty data is accepted,
    /// // this data (50 bytes) generates statistically weaker hash
    /// // (and thus rejected by default).
    /// // This is even stronger failure than a half-empty buckets.
    /// // The failure is caused by an *extremely* unbalanced data distribution.
    /// generator.update(b"ABCDEABCDEABCDEABCDEABCDEABCDEABCDEABCDEABCDEABCDE");
    /// let result = generator.finalize_with_options(
    ///     GeneratorOptions::new().allow_statistically_weak_buckets_half(true)
    /// );
    /// assert!(result.is_err());
    /// let err = result.unwrap_err();
    /// assert_eq!(err.category(), GeneratorErrorCategory::DataDistribution);
    ///
    /// // But with extended permissive mode, it succeeds
    /// // (but you can see that there are too many zeroes which will make
    /// //  the comparison less useful).
    /// let hash = generator.finalize_with_options(
    ///     GeneratorOptions::new().allow_statistically_weak_buckets_quarter(true)
    /// ).unwrap();
    /// let expected = "T14590440C330003C00C0033000000C300F000C00300C030000000C3000000000000C000";
    /// let expected = Tlsh::from_str(expected).unwrap();
    /// assert_eq!(hash.to_string(), expected.to_string());
    /// ```
    pub fn allow_statistically_weak_buckets_quarter(&mut self, value: bool) -> &mut Self {
        self.incompat_flags.set(
            TLSHIncompatibleGeneratorFlags::ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER,
            value,
        );
        self
    }
}
impl Default for GeneratorOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// The public part for later `pub use` at crate root.
pub(crate) mod public {
    use super::*;

    /// The trait to represent a fuzzy hash generator.
    ///
    /// This trait is implemented by [`Generator`].
    pub trait GeneratorType {
        /// The output type.
        type Output: FuzzyHashType;

        /// Whether the checksum is updated by this generator type.
        ///
        /// If this type is [`false`], the resulting fuzzy hash from this
        /// generator will have checksum part with all zeroes.
        ///
        /// In the official TLSH implementation, it is always [`true`]
        /// except multi-threaded and private modes.  This crate currently
        /// does not support those modes but will be implemented in the future.
        const IS_CHECKSUM_EFFECTIVE: bool;

        /// The minimum data length
        /// (on [all modes](DataLengthProcessingMode)).
        const MIN: u32;

        /// The minimum data length
        /// (on [the conservative mode](DataLengthProcessingMode::Conservative)).
        const MIN_CONSERVATIVE: u32;

        /// The maximum data length (inclusive).
        const MAX: u32;

        /// Returns the data length it processed.
        ///
        /// If the generator is unable to represent exact data length it
        /// processed, it returns [`None`].  Otherwise, the exact data length is
        /// returned by [`Some`].
        fn processed_len(&self) -> Option<u32>;

        /// Update the generator by feeding data to it.
        fn update(&mut self, data: &[u8]);

        /// Finalize the fuzzy hash with specified options.
        ///
        /// You will likely use the default options and use
        /// [`finalize()`](Self::finalize()) instead.
        fn finalize_with_options(
            &self,
            options: &GeneratorOptions,
        ) -> Result<Self::Output, GeneratorError>;

        /// Finalize the fuzzy hash with the default options.
        ///
        /// If you want to use [a custom generator options](GeneratorError),
        /// use [`finalize_with_options()`](Self::finalize_with_options())
        /// instead.
        #[inline(always)]
        fn finalize(&self) -> Result<Self::Output, GeneratorError> {
            self.finalize_with_options(&Default::default())
        }

        /// Tests: count non-zero buckets.
        #[cfg(test)]
        fn count_nonzero_buckets(&self) -> usize;
    }
}

/// The inner representation and its implementation.
pub(crate) mod inner {
    use super::*;

    /// The fuzzy hash generator corresponding specified parameters.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Generator<
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
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        /// The buckets to store local features.
        pub(super) buckets: FuzzyHashBucketsData<SIZE_BUCKETS>,

        /// The total length of the input *after we finish filling*
        /// [`tail`](Self::tail).
        ///
        /// We have to add [`tail_len`](Self::tail_len) to get the minimum
        /// length we processed because it excludes the length of
        /// [`tail`](Self::tail).
        pub(super) len: u32,

        /// The checksum determined from the data (and number of buckets).
        pub(super) checksum: FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>,

        /// Previous (last) bytes processed.
        ///
        /// Physical size of this array is [`TAIL_SIZE`](Self::TAIL_SIZE) which
        /// is equal to one less than [`WINDOW_SIZE`].
        ///
        /// This is because we'll process the file by a sliding window of the
        /// size [`WINDOW_SIZE`].  For instance, the first processed window is
        /// the contents of this array plus the first byte (the total length is
        /// [`WINDOW_SIZE`]).
        ///
        /// The effective length is handled separately by
        /// [`tail_len`](Self::tail_len).
        pub(super) tail: [u8; WINDOW_SIZE - 1],

        /// The effective length of [`tail`](Self::tail).
        ///
        /// If we haven't processed enough number of bytes yet, this is smaller
        /// than the length of [`tail`](Self::tail) and we have to wait more
        /// data to be fed.
        pub(super) tail_len: u32,
    }

    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Generator<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper<
            RawBodyType = [u8; SIZE_BODY],
            RawBucketType = [u32; SIZE_BUCKETS],
        >,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        /// The maximum length of [`tail`](Self::tail) which is equal to one
        /// less than [`WINDOW_SIZE`].
        ///
        /// If [`tail_len`](Self::tail_len) gets to this value and we have more
        /// bytes to process, we start processing the file using
        /// [`WINDOW_SIZE`]-byte sliding window.
        const TAIL_SIZE: u32 = (WINDOW_SIZE - 1) as u32;

        /// The maximum [`len`](Self::len), which is equal to the value first
        /// overflows [`u32`] if we calculate `len + tail_len`.
        const MAX_LEN: u32 = u32::MAX - (Self::TAIL_SIZE - 1);

        /// TLSH's B (bucket) mapping suitable for this generator.
        #[inline(always)]
        fn b_mapping(v0: u8, v1: u8, v2: u8, v3: u8) -> u8 {
            FuzzyHashBucketsInfo::<SIZE_BUCKETS>::b_mapping(v0, v1, v2, v3)
        }
    }
    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > Default
        for Generator<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper<
            RawBodyType = [u8; SIZE_BODY],
            RawBucketType = [u32; SIZE_BUCKETS],
        >,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        fn default() -> Self {
            Self {
                buckets: FuzzyHashBucketsData::new(),
                len: 0,
                checksum: FuzzyHashChecksumData::new(),
                tail: [0; WINDOW_SIZE - 1],
                tail_len: 0,
            }
        }
    }
    impl<
            const SIZE_CKSUM: usize,
            const SIZE_BODY: usize,
            const SIZE_BUCKETS: usize,
            const SIZE_IN_BYTES: usize,
            const SIZE_IN_STR_BYTES: usize,
        > crate::GeneratorType
        for Generator<SIZE_CKSUM, SIZE_BODY, SIZE_BUCKETS, SIZE_IN_BYTES, SIZE_IN_STR_BYTES>
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
        FuzzyHashBucketsInfo<SIZE_BUCKETS>: FuzzyHashBucketMapper<
            RawBodyType = [u8; SIZE_BODY],
            RawBucketType = [u32; SIZE_BUCKETS],
        >,
        FuzzyHashChecksumData<SIZE_CKSUM, SIZE_BUCKETS>: FuzzyHashChecksum,
        VerboseFuzzyHashParams<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >: ConstrainedVerboseFuzzyHashParams,
        LengthProcessingInfo<SIZE_BUCKETS>: ConstrainedLengthProcessingInfo,
    {
        type Output = crate::hash::inner::FuzzyHash<
            SIZE_CKSUM,
            SIZE_BODY,
            SIZE_BUCKETS,
            SIZE_IN_BYTES,
            SIZE_IN_STR_BYTES,
        >;

        const IS_CHECKSUM_EFFECTIVE: bool = true;
        const MIN: u32 = LengthProcessingInfo::<SIZE_BUCKETS>::MIN;
        const MIN_CONSERVATIVE: u32 = LengthProcessingInfo::<SIZE_BUCKETS>::MIN_CONSERVATIVE;
        const MAX: u32 = LengthProcessingInfo::<SIZE_BUCKETS>::MAX;

        fn processed_len(&self) -> Option<u32> {
            self.len.checked_add(self.tail_len)
        }

        fn update(&mut self, data: &[u8]) {
            // Fill self.tail (before we start updating).
            let mut data = data;
            if self.tail_len < Self::TAIL_SIZE {
                let tail_len = self.tail_len as usize;
                let remaining = Self::TAIL_SIZE as usize - tail_len;
                if data.len() <= remaining {
                    self.tail[tail_len..tail_len + data.len()].copy_from_slice(data);
                    self.tail_len += data.len() as u32;
                    // self.tail is not yet filled
                    // (or filled but no more bytes to update).
                    return;
                }
                self.tail[tail_len..].copy_from_slice(&data[..remaining]);
                self.tail_len += remaining as u32;
                // self.tail is now filled and we have more data. Continuing.
                data = &data[remaining..];
            }
            // If we have processed 4GiB already, ignore the rest.
            optionally_unsafe! {
                invariant!(Self::TAIL_SIZE > 0);
            }
            if unlikely(self.len >= Self::MAX_LEN) {
                return;
            }
            // Update the processed data length
            let mut data_len = u32::try_from(data.len()).unwrap_or(u32::MAX);
            if unlikely(data_len > Self::MAX_LEN - self.len) {
                // Processing the data exceeds the first 4GiB.
                data_len = Self::MAX_LEN - self.len;
                data = &data[..data_len as usize];
            }
            self.len += data_len;
            // Update the buckets based on the 5-byte window.
            let (mut b0, mut b1, mut b2, mut b3) =
                (self.tail[0], self.tail[1], self.tail[2], self.tail[3]);
            for &b4 in data {
                // Update the checksum and buckets
                self.checksum.update(b4, b3);
                self.buckets.increment(Self::b_mapping(0x2, b4, b3, b2));
                self.buckets.increment(Self::b_mapping(0x3, b4, b3, b1));
                self.buckets.increment(Self::b_mapping(0x5, b4, b2, b1));
                self.buckets.increment(Self::b_mapping(0x7, b4, b2, b0));
                self.buckets.increment(Self::b_mapping(0xb, b4, b3, b0));
                self.buckets.increment(Self::b_mapping(0xd, b4, b1, b0));
                // Shift
                (b0, b1, b2, b3) = (b1, b2, b3, b4);
            }
            // Update self.tail.
            if likely(data.len() >= self.tail.len()) {
                // Full overwrite
                self.tail
                    .copy_from_slice(&data[data.len() - Self::TAIL_SIZE as usize..]);
            } else {
                // Partial overwrite (shift and write)
                self.tail.copy_within(data.len().., 0);
                self.tail[(Self::TAIL_SIZE as usize) - data.len()..].copy_from_slice(data);
            }
        }

        fn finalize_with_options(
            &self,
            options: &GeneratorOptions,
        ) -> Result<Self::Output, GeneratorError> {
            let len = self.processed_len().unwrap_or(u32::MAX); // assume u32::MAX is an invalid value.
            let validity = DataLengthValidity::new::<SIZE_BUCKETS>(len);
            if validity.is_err_on(options.length_mode) {
                match validity {
                    DataLengthValidity::TooLarge => {
                        return Err(GeneratorError::TooLargeInput);
                    }
                    _ => {
                        if !options
                            .incompat_flags
                            .contains(TLSHIncompatibleGeneratorFlags::ALLOW_SMALL_SIZE_FILES)
                        {
                            return Err(GeneratorError::TooSmallInput);
                        }
                    }
                }
            }
            // Get encoded length part.
            let lvalue = FuzzyHashLengthEncoding::new(len).unwrap();
            // Get quartile values and number of non-zero buckets.
            let buckets: [u32; SIZE_BUCKETS] = self.buckets.data().try_into().unwrap();
            let nonzero_count = buckets.iter().filter(|&&x| x != 0).count();
            let mut copy_buckets = buckets;
            let (l0, &mut mut q2, l1) = copy_buckets.select_nth_unstable(SIZE_BUCKETS / 2 - 1);
            let (_, &mut mut q1, _) = l0.select_nth_unstable(SIZE_BUCKETS / 4 - 1);
            let (_, &mut mut q3, _) = l1.select_nth_unstable(SIZE_BUCKETS / 4 - 1);
            // Reject if the data distribution is too statistically unbalanced
            // (so that an attempt to calculate Q ratios will cause an issue)
            // unless an option is specified
            // (in this case, dummy quartile values are set).
            if q3 == 0 {
                if !options.incompat_flags.contains(
                    TLSHIncompatibleGeneratorFlags::ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER,
                ) {
                    return Err(GeneratorError::BucketsAreThreeQuarterEmpty);
                }
                // Set a value to force outputting a fuzzy hash.
                (q1, q2, q3) = (1, 1, 1);
            }
            // Reject if the data distribution is statistically unbalanced
            // unless an option is specified.
            if nonzero_count < FuzzyHashBucketsInfo::<SIZE_BUCKETS>::MIN_NONZERO_BUCKETS
                && !options.incompat_flags.intersects(
                    TLSHIncompatibleGeneratorFlags::ALLOW_STATISTICALLY_WEAK_BUCKETS_HALF
                        | TLSHIncompatibleGeneratorFlags::ALLOW_STATISTICALLY_WEAK_BUCKETS_QUARTER,
                )
            {
                return Err(GeneratorError::BucketsAreHalfEmpty);
            }
            // Get the Q ratios.
            let (q1ratio, q2ratio) = if options
                .compat_flags
                .contains(TLSHCompatibleGeneratorFlags::PURE_INTEGER_QRATIO_COMPUTATION)
            {
                (
                    (((q1 as u64 * 100) / q3 as u64) % 16) as u8,
                    (((q2 as u64 * 100) / q3 as u64) % 16) as u8,
                )
            } else {
                (
                    (((q1.wrapping_mul(100) as f32) / q3 as f32) as u32 % 16) as u8,
                    (((q2.wrapping_mul(100) as f32) / q3 as f32) as u32 % 16) as u8,
                )
            };
            let qratios = FuzzyHashQRatios::new(q1ratio, q2ratio);
            // Compute the body part.
            let mut body = [0u8; SIZE_BODY];
            FuzzyHashBucketsInfo::<SIZE_BUCKETS>::aggregate_buckets(
                &mut body, &buckets, q1, q2, q3,
            );
            // Return the new fuzzy hash object.
            Ok(Self::Output::from_raw(
                FuzzyHashBodyData::from_raw(body),
                self.checksum,
                lvalue,
                qratios,
            ))
        }

        #[cfg(test)]
        fn count_nonzero_buckets(&self) -> usize {
            // Excerpt from finalize_with_options above.
            let buckets: [u32; SIZE_BUCKETS] = self.buckets.data().try_into().unwrap();
            buckets.iter().filter(|&&x| x != 0).count()
        }
    }
}

/// The macro representing the inner generator type.
macro_rules! inner_type {
    ($ty:ty) => {
        <<$ty as ConstrainedFuzzyHashType>::Params as ConstrainedFuzzyHashParams>::InnerGeneratorType
    };
}

/// The fuzzy hash generator corresponding specified fuzzy hash type.
///
/// For the main functionalities, see [`GeneratorType`] documentation.
#[derive(Debug, Clone)]
pub struct Generator<T: ConstrainedFuzzyHashType> {
    /// The inner object representing actual contents of the generator.
    pub(crate) inner:
        <<T as ConstrainedFuzzyHashType>::Params as ConstrainedFuzzyHashParams>::InnerGeneratorType,
}
impl<T: ConstrainedFuzzyHashType> Generator<T> {
    /// Creates the new generator.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}
impl<T: ConstrainedFuzzyHashType> Default for Generator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: ConstrainedFuzzyHashType> GeneratorType for Generator<T> {
    type Output = T;

    const IS_CHECKSUM_EFFECTIVE: bool = <inner_type!(T)>::IS_CHECKSUM_EFFECTIVE;
    const MIN: u32 = <inner_type!(T)>::MIN;
    const MIN_CONSERVATIVE: u32 = <inner_type!(T)>::MIN_CONSERVATIVE;
    const MAX: u32 = <inner_type!(T)>::MAX;

    #[inline(always)]
    fn processed_len(&self) -> Option<u32> {
        self.inner.processed_len()
    }

    #[inline(always)]
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    #[inline(always)]
    fn finalize_with_options(
        &self,
        options: &GeneratorOptions,
    ) -> Result<Self::Output, GeneratorError> {
        self.inner.finalize_with_options(options).map(T::new)
    }

    #[cfg(test)]
    fn count_nonzero_buckets(&self) -> usize {
        self.inner.count_nonzero_buckets()
    }
}

pub(crate) mod tests;
