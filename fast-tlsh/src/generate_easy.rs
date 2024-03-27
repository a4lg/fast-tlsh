// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The easy wrapper for generator functionalities.

#![cfg(feature = "easy-functions")]

use crate::errors::GeneratorError;
use crate::generate::Generator;
use crate::params::ConstrainedFuzzyHashType;
use crate::{GeneratorType, Tlsh};

/// Generates a fuzzy hash from a given buffer
/// (with specified output type).
///
/// # Example
///
/// ```
/// type CustomTlsh = tlsh::hashes::Short;
///
/// // Make a fuzzy hash from the buffer.
/// let hash = tlsh::hash_buf_for::<CustomTlsh>(b"Hello, World!").unwrap();
///
/// // Compare with known result.
/// // Note: short fuzzy hashes accept very short inputs.
/// assert_eq!(hash.to_string(), "T1E16004017D3551777571D55C005CC5");
/// ```
pub fn hash_buf_for<T: ConstrainedFuzzyHashType>(buffer: &[u8]) -> Result<T, GeneratorError> {
    let mut generator = Generator::<T>::new();
    generator.update(buffer);
    generator.finalize()
}

/// Generates a fuzzy hash from a given buffer.
///
/// # Example
///
/// ```
/// // Make a fuzzy hash from the buffer.
/// let hash = tlsh::hash_buf(
///     b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
///     eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad \
///     minim veniam, quis nostrud exercitation ullamco laboris nisi ut \
///     aliquip ex ea commodo consequat. Duis aute irure dolor in \
///     reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
///     pariatur. Excepteur sint occaecat cupidatat non proident, sunt in \
///     culpa qui officia deserunt mollit anim id est laborum.",
/// )
/// .unwrap();
///
/// // Compare with known result.
/// assert_eq!(
///     hash.to_string(),
///     "T1DCF0DC36520C1B007FD32079B226559FD998A0200725E75AFCEAC99F5881184A4B1AA2"
/// );
/// ```
pub fn hash_buf(buffer: &[u8]) -> Result<Tlsh, GeneratorError> {
    hash_buf_for::<Tlsh>(buffer)
}

mod tests;
