// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

// Separate from README.md to use rustdoc-specific features in docs/readme.md.
#![doc = include_str!("_docs/readme.md")]
// no_std
#![cfg_attr(not(any(test, doc, feature = "std")), no_std)]
// Regular nightly features
#![cfg_attr(feature = "unstable", feature(coverage_attribute))]
#![cfg_attr(feature = "unstable", feature(doc_cfg))]
#![cfg_attr(feature = "unstable", feature(likely_unlikely))]
#![cfg_attr(feature = "unstable", feature(portable_simd))]
#![cfg_attr(
    all(feature = "unstable", target_arch = "arm"),
    feature(arm_target_feature)
)]
#![cfg_attr(
    all(feature = "unstable", target_arch = "arm"),
    feature(stdarch_arm_feature_detection)
)]
#![cfg_attr(
    all(feature = "unstable", target_arch = "arm"),
    feature(stdarch_arm_neon_intrinsics)
)]
// In the code maintenance mode, disallow all warnings.
#![cfg_attr(feature = "maint-code", deny(warnings))]
// Unsafe code is *only* allowed on enabling either arch-specific SIMD
// ("simd-per-arch") or "unsafe" features ("simd-per-arch" and "simd-portable"
// features only indicate those "implemented using SIMD inside this crate").
// If only arch-specific SIMD features are enabled,
// such code requires explicit allow.
#![cfg_attr(
    not(any(feature = "simd-per-arch", feature = "unsafe")),
    forbid(unsafe_code)
)]
#![cfg_attr(
    all(feature = "simd-per-arch", not(feature = "unsafe")),
    deny(unsafe_code)
)]
// Non-test code requires documents (including private items)
#![cfg_attr(not(test), warn(missing_docs))]
#![cfg_attr(not(test), warn(clippy::missing_docs_in_private_items))]
// Unless in the maintenance mode, allow unknown lints.
#![cfg_attr(not(feature = "maint-lints"), allow(unknown_lints))]
// Unless in the maintenance mode, allow old lint names.
#![cfg_attr(not(feature = "maint-lints"), allow(renamed_and_removed_lints))]
// Tests: allow unused unsafe blocks (invariant! does will not need unsafe
// on tests but others may need this macro).
#![cfg_attr(test, allow(unused_unsafe))]
// Tests: non-simplified boolean expressions should be allowed.
#![cfg_attr(test, allow(clippy::nonminimal_bool))]
// Tests: assertion on constants should be allowed.
#![cfg_attr(test, allow(clippy::assertions_on_constants))]
// Tests: redundant clones should be allowed.
#![cfg_attr(test, allow(clippy::redundant_clone))]

// alloc is required when the "alloc" feature is enabled or testing (including doctests).
#[cfg(any(feature = "alloc", test, doc))]
extern crate alloc;

mod internals;

pub mod _docs;
pub mod buckets;
mod compare;
mod errors;
pub mod generate;
pub mod hash;
pub mod hashes;
pub mod length;
mod params;
mod parse;

mod compare_easy;
mod generate_easy;
mod generate_easy_std;

// Easy function re-exports
#[cfg(feature = "easy-functions")]
pub use compare_easy::{compare, compare_with};
#[cfg(feature = "easy-functions")]
pub use generate_easy::{hash_buf, hash_buf_for};
#[cfg(all(feature = "easy-functions", feature = "std"))]
pub use generate_easy_std::{hash_file, hash_file_for, hash_stream, hash_stream_for};

// Trait re-exports
pub use generate::public::GeneratorType;
pub use hash::public::FuzzyHashType;

// Type re-exports
pub use compare::ComparisonConfiguration;
pub use errors::{GeneratorError, GeneratorErrorCategory};
pub use errors::{OperationError, ParseError};
pub use generate::GeneratorOptions;
pub use hash::HexStringPrefix;
pub use length::DataLengthProcessingMode;

#[cfg(all(feature = "easy-functions", feature = "std"))]
pub use errors::GeneratorOrIOError;
#[cfg(feature = "easy-functions")]
pub use errors::{ParseErrorEither, ParseErrorSide};

/// The default fuzzy hash type.
pub type Tlsh = hashes::Normal;

/// The fuzzy hash generator with the default parameter.
pub type TlshGenerator = generate::Generator<Tlsh>;

/// The fuzzy hash generator with specified parameter
/// (or output fuzzy hash type).
///
/// The type parameter `T` must be either:
///
/// *   [`Tlsh`], the default (in this case, you'd better to use
///     [`TlshGenerator`]),
/// *   One of the types in [`hashes`] (each represents a fuzzy hash type
///     and its parameter at the same time) or
/// *   [`hash::FuzzyHash`] with valid parameters.
///
/// unless you have something pointing to these types above somewhere else.
///
/// # Example
///
/// ```
/// use tlsh::prelude::*;
///
/// // Initialize a generator for long fuzzy hash (with 256 buckets)
/// // with long (3-byte) checksum.
/// let mut generator = TlshGeneratorFor::<tlsh::hashes::LongWithLongChecksum>::new();
/// ```
pub type TlshGeneratorFor<T> = generate::Generator<T>;

/// The recommended set (prelude) to import.
///
/// It provides a subset of crate-root types, traits and type aliases
/// suitable for using this crate.  Because some methods require importing
/// certain traits, just importing this can be convenient (not to confuse
/// beginners, those traits are imported as `_`).
///
/// It (intentionally) excludes crate-root easy functions because
/// it's not a big cost to type `tlsh::`.
///
/// It also excludes [`tlsh::hashes`](crate::hashes) to avoid confusion.
pub mod prelude {
    pub use super::FuzzyHashType as _;
    pub use super::GeneratorType as _;

    pub use super::Tlsh;
    pub use super::{TlshGenerator, TlshGeneratorFor};
}

mod tests;
