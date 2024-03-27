// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Easy comparison for two TLSH strings.

#![cfg(feature = "easy-functions")]

use crate::errors::{ParseErrorEither, ParseErrorSide};
use crate::params::ConstrainedFuzzyHashType;
use crate::Tlsh;

/// Compare two fuzzy hashes with specified intermediate fuzzy hash type.
///
/// If a parse error occurs, [`Err`] containing
/// [a parse error](ParseErrorEither) is returned.  Otherwise, [`Ok`] containing
/// the distance-based score is returned.
///
/// # Examples
///
/// ```
/// type CustomTlsh = tlsh::hashes::Short;
///
/// // Distance between rustc 1.66.1–1.67.1 (Linux, x86_64) is 2
/// // on the short variant of TLSH.
/// let result = tlsh::compare_with::<CustomTlsh>(
///     "T140D5F17F44F8AB007AE2AC46E515DC",
///     "T140D5F17F44FCAB007AE2A846E515DC"
/// );
/// assert_eq!(result, Ok(2));
/// ```
///
/// ```
/// use tlsh::errors::{ParseError, ParseErrorEither, ParseErrorSide};
///
/// type CustomTlsh = tlsh::hashes::Short;
///
/// // The parser fails on the right.
/// let result = tlsh::compare_with::<CustomTlsh>(
///     "T140D5F17F44F8AB007AE2AC46E515DC",
///     "TNULL"
/// );
/// let err = result.unwrap_err();
/// assert_eq!(err.side(), ParseErrorSide::Right);
/// assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
/// ```
pub fn compare_with<T: ConstrainedFuzzyHashType>(
    lhs: &str,
    rhs: &str,
) -> Result<u32, ParseErrorEither> {
    let lhs: T = match str::parse(lhs) {
        Ok(value) => value,
        Err(err) => {
            return Err(ParseErrorEither(ParseErrorSide::Left, err));
        }
    };
    let rhs: T = match str::parse(rhs) {
        Ok(value) => value,
        Err(err) => {
            return Err(ParseErrorEither(ParseErrorSide::Right, err));
        }
    };
    Ok(lhs.compare(&rhs))
}

/// Compare two fuzzy hashes.
///
/// If a parse error occurs, [`Err`] containing
/// [a parse error](ParseErrorEither) is returned.  Otherwise, [`Ok`] containing
/// the distance-based score (`0..=2473`) is returned.
///
/// # Examples
///
/// ```
/// // Distance between rustc 1.66.1–1.67.1 (Linux, x86_64) is 9.
/// let result = tlsh::compare(
///     "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488",
///     "T1EDD5BE96FFE41D1BCC268C7699AE4720B7B2A0032716DBAF1848A7647DD77C0DF16488"
/// );
/// assert_eq!(result, Ok(9));
/// ```
///
/// ```
/// use tlsh::errors::{ParseError, ParseErrorEither, ParseErrorSide};
///
/// // The parser fails on the right.
/// let result = tlsh::compare(
///     "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488",
///     "TNULL"
/// );
/// let err = result.unwrap_err();
/// assert_eq!(err.side(), ParseErrorSide::Right);
/// assert_eq!(err.inner_err(), ParseError::InvalidStringLength);
/// ```
#[inline(always)]
pub fn compare(lhs: &str, rhs: &str) -> Result<u32, ParseErrorEither> {
    compare_with::<Tlsh>(lhs, rhs)
}

mod tests;
