// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The TLSH parameters.

use crate::internals::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};
use crate::internals::hash::checksum::{CHECKSUM_SIZE_LONG, CHECKSUM_SIZE_NORMAL};
use crate::{FuzzyHashType, GeneratorType};

/// The private part.
mod private {
    /// The sealed trait for verbose parameters.
    pub trait SealedVerboseParam {}
    /// The sealed trait for parameters.
    pub trait SealedParam {}
    /// The sealed trait for constrained fuzzy hashes.
    pub trait SealedFuzzyHashes {}
}

/// A marker struct for fuzzy hashing parameters.
pub struct FuzzyHashParams<const SIZE_CKSUM: usize, const SIZE_BUCKETS: usize>;

/// An adapter trait for valid fuzzy hashing parameters.
pub trait ConstrainedFuzzyHashParams: private::SealedParam {
    /// The inner fuzzy hash type used by the public implementation.
    ///
    /// This is an instantiation of
    /// [`FuzzyHash`](crate::internals::hash::FuzzyHash).
    type InnerFuzzyHashType: FuzzyHashType
        + core::fmt::Debug
        + core::fmt::Display
        + Clone
        + Copy
        + PartialEq
        + Eq;
    /// The inner generator type used by the public implementation.
    ///
    /// This is an instantiation of
    /// [`Generator`](crate::internals::generate::Generator).
    type InnerGeneratorType: GeneratorType<Output = Self::InnerFuzzyHashType>
        + core::fmt::Debug
        + Default
        + Clone;
}

/// An adapter trait for valid public fuzzy hash types.
pub trait ConstrainedFuzzyHashType:
    private::SealedFuzzyHashes
    + core::fmt::Debug
    + core::fmt::Display
    + FuzzyHashType
    + Clone
    + PartialEq
    + Eq
{
    /// The parameters corresponding the type.
    type Params: ConstrainedFuzzyHashParams;
    /// Creates an object from the inner object.
    fn new(inner: <Self::Params as ConstrainedFuzzyHashParams>::InnerFuzzyHashType) -> Self;
}

/// A marker struct for fuzzy hashing parameters (verbose).
pub struct VerboseFuzzyHashParams<
    const SIZE_CKSUM: usize,
    const SIZE_BODY: usize,
    const SIZE_BUCKETS: usize,
    const SIZE_IN_BYTES: usize,
    const SIZE_IN_STR_BYTES: usize,
>;

/// A marker trait for valid fuzzy hashing parameters (verbose).
pub trait ConstrainedVerboseFuzzyHashParams: private::SealedVerboseParam {}
impl<T> ConstrainedVerboseFuzzyHashParams for T where T: private::SealedVerboseParam {}

/// The macro to convert symbolic buckets constant name to string.
macro_rules! param_buckets_desc {
    (NUM_BUCKETS_SHORT) => {
        "Short"
    };
    (NUM_BUCKETS_NORMAL) => {
        "Normal"
    };
    (NUM_BUCKETS_LONG) => {
        "Long"
    };
}

/// The macro to convert symbolic buckets constant name to the official name.
macro_rules! param_buckets_desc_alt {
    (NUM_BUCKETS_SHORT) => {
        "min hash"
    };
    (NUM_BUCKETS_NORMAL) => {
        "compact hash"
    };
    (NUM_BUCKETS_LONG) => {
        "full hash"
    };
}

/// The macro to convert symbolic checksum constant name to string.
macro_rules! param_checksum_desc {
    (CHECKSUM_SIZE_NORMAL) => {
        "1-byte"
    };
    (CHECKSUM_SIZE_LONG) => {
        "3-byte"
    };
}

/// The inner fuzzy hash type.
macro_rules! inner_fuzzy_hash_type {
    ($size_checksum:expr, $size_buckets:tt) => {
        $crate::internals::hash::FuzzyHash<
            {$size_checksum},
            {$size_buckets / 4},
            {$size_buckets},
            {$size_buckets / 4 + 2 + $size_checksum},
            {($size_buckets / 4 + 2 + $size_checksum) * 2 + 2}
        >
    };
}

/// The inner generator type.
macro_rules! inner_generator_type {
    ($size_checksum:expr, $size_buckets:tt) => {
        $crate::internals::generate::Generator<
            {$size_checksum},
            {$size_buckets / 4},
            {$size_buckets},
            {$size_buckets / 4 + 2 + $size_checksum},
            {($size_buckets / 4 + 2 + $size_checksum) * 2 + 2}
        >
    };
}

/// The fuzzy hash parameter template generator.
macro_rules! params {
    {$($name:ident = ($size_checksum:tt, $size_buckets:tt);)*} => {
        $(
            impl private::SealedParam
                for FuzzyHashParams<{$size_checksum}, {$size_buckets}>
            {
            }
            impl private::SealedVerboseParam
                for VerboseFuzzyHashParams<
                    {$size_checksum},
                    {$size_buckets / 4},
                    {$size_buckets},
                    {$size_buckets / 4 + 2 + $size_checksum},
                    {($size_buckets / 4 + 2 + $size_checksum) * 2 + 2}
                >
            {
            }
            impl ConstrainedFuzzyHashParams for FuzzyHashParams<{$size_checksum}, {$size_buckets}> {
                type InnerFuzzyHashType = inner_fuzzy_hash_type!($size_checksum, $size_buckets);
                type InnerGeneratorType = inner_generator_type!($size_checksum, $size_buckets);
            }
            impl private::SealedFuzzyHashes
                for crate::hash::FuzzyHash<{$size_checksum}, {$size_buckets}>
            {
            }
            impl ConstrainedFuzzyHashType for crate::hash::FuzzyHash<{$size_checksum}, {$size_buckets}> {
                type Params = FuzzyHashParams<{$size_checksum}, {$size_buckets}>;
                fn new(inner: <Self::Params as ConstrainedFuzzyHashParams>::InnerFuzzyHashType) -> Self {
                    Self::new(inner)
                }
            }
        )*
        /// Fuzzy hash types for later re-exports.
        pub(crate) mod exported_hashes {
            use super::*;
            $(
                #[doc = concat!(
                    param_buckets_desc!($size_buckets),
                    " fuzzy hash type (",
                    param_buckets_desc_alt!($size_buckets),
                    ") with ",
                    param_checksum_desc!($size_checksum),
                    " checksum.\n",
                    "\n",
                    "For more information about the implementation, see ",
                    "[`FuzzyHashType`](crate::FuzzyHashType).\n",
                    "\n",
                    "For other types with different parameters, ",
                    "see the [module documentation](crate::hashes)."
                )]
                pub type $name =
                    crate::hash::FuzzyHash<{$size_checksum}, {$size_buckets}>;
            )*
        }
    };
}
params! {
    Short                  = (CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_SHORT);
    Normal                 = (CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_NORMAL);
    NormalWithLongChecksum = (CHECKSUM_SIZE_LONG,   NUM_BUCKETS_NORMAL);
    Long                   = (CHECKSUM_SIZE_NORMAL, NUM_BUCKETS_LONG);
    LongWithLongChecksum   = (CHECKSUM_SIZE_LONG,   NUM_BUCKETS_LONG);
}

mod tests;
