// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Types representing specific types of errors.

use core::fmt::{Display, Formatter, Result};

/// An error type representing an error (generally) while parsing a fuzzy hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseError {
    /// The data length field is too large.
    ///
    /// On the string parser or deserializer, this error will not be generated
    /// unless the strict parser is enabled.
    LengthIsTooLarge,
    /// Invalid prefix is encountered.
    ///
    /// This type of error is generated when we need a prefix but cannot find
    /// a valid one (TLSHv1 `"T1"`).
    InvalidPrefix,
    /// An invalid character is encountered.
    InvalidCharacter,
    /// The string length is invalid.
    InvalidStringLength,
    /// The checksum part contained an invalid value.
    InvalidChecksum,
}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            ParseError::LengthIsTooLarge => "length field is too large",
            ParseError::InvalidPrefix => "encountered an invalid prefix",
            ParseError::InvalidCharacter => "encountered an invalid character",
            ParseError::InvalidStringLength => "string length is invalid",
            ParseError::InvalidChecksum => "has an invalid checksum field",
        })
    }
}
#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
#[cfg(all(not(feature = "std"), fast_tlsh_error_in_core = "stable"))]
impl core::error::Error for ParseError {}

/// An error type representing an error (generally) while processing a fuzzy hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OperationError {
    /// The buffer you specified is too small to finish the operation successfully.
    ///
    /// The reason of this error type is because the buffer (slice) you have
    /// specified is too small for the output format you requested.
    BufferIsTooSmall,
}
impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            OperationError::BufferIsTooSmall => "buffer is too small to store the result",
        })
    }
}
#[cfg(feature = "std")]
impl std::error::Error for OperationError {}
#[cfg(all(not(feature = "std"), fast_tlsh_error_in_core = "stable"))]
impl core::error::Error for OperationError {}

/// An error category type for [a generator error](GeneratorError).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneratorErrorCategory {
    /// The error is (mainly) about the length of the data.
    DataLength,

    /// The error is (mainly) about the distribution of the data
    /// (or, repetitiveness).
    DataDistribution,
}

/// An error type representing an error while generating a fuzzy hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneratorError {
    /// The input data is too large to process.
    TooLargeInput,
    /// The input data is too small to process.
    ///
    /// Whether the input data is too small is normally determined by the value
    /// of [`DataLengthProcessingMode`](crate::length::DataLengthProcessingMode).
    ///
    /// If we prefer compatibility with the original TLSH implementation,
    /// we cannot generate a fuzzy hash from the data smaller than 50 bytes.
    TooSmallInput,
    /// Too many buckets (roughly half or more) are empty.
    ///
    /// This error indicates the input data is either too small or too
    /// repetitive so that enough number of buckets cannot be filled
    /// (i.e. even if we force to output a fuzzy hash, the result might be
    /// statistically unreliable).
    BucketsAreHalfEmpty,
    /// Too many buckets (roughly 3/4 or more) are empty.
    ///
    /// This is similar to [`BucketsAreHalfEmpty`](Self::BucketsAreHalfEmpty)
    /// but indicates more extreme statistic distribution so that computing
    /// a Q ratio will result in a division by zero.
    BucketsAreThreeQuarterEmpty,
}
impl GeneratorError {
    /// Retrieves the category of the generator error.
    pub fn category(&self) -> GeneratorErrorCategory {
        match *self {
            GeneratorError::TooLargeInput => GeneratorErrorCategory::DataLength,
            GeneratorError::TooSmallInput => GeneratorErrorCategory::DataLength,
            GeneratorError::BucketsAreHalfEmpty => GeneratorErrorCategory::DataDistribution,
            GeneratorError::BucketsAreThreeQuarterEmpty => GeneratorErrorCategory::DataDistribution,
        }
    }
}
impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            GeneratorError::TooLargeInput => "input data is too large to process",
            GeneratorError::TooSmallInput => "input data is too small to process",
            GeneratorError::BucketsAreHalfEmpty => {
                "approximately half or more effective buckets are empty"
            }
            GeneratorError::BucketsAreThreeQuarterEmpty => {
                "approximately 3/4 or more effective buckets are empty"
            }
        })
    }
}
#[cfg(feature = "std")]
impl std::error::Error for GeneratorError {}
#[cfg(all(not(feature = "std"), fast_tlsh_error_in_core = "stable"))]
impl core::error::Error for GeneratorError {}

/// The operand (side) which caused a parse error.
#[cfg(feature = "easy-functions")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorSide {
    /// The left hand side.
    Left,
    /// The right hand side.
    Right,
}

/// The error type representing a parse error for one of the operands
/// specified to the [`compare()`](crate::compare()) function.
#[cfg(feature = "easy-functions")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseErrorEither(pub(crate) ParseErrorSide, pub(crate) ParseError);
#[cfg(feature = "easy-functions")]
impl ParseErrorEither {
    /// Returns which operand caused a parse error.
    pub fn side(&self) -> ParseErrorSide {
        self.0
    }

    /// Returns the inner error.
    pub fn inner_err(&self) -> ParseError {
        self.1
    }
}
#[cfg(feature = "easy-functions")]
impl Display for ParseErrorEither {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "error occurred while parsing fuzzy hash {side} ({msg})",
            side = match self.side() {
                ParseErrorSide::Left => 1,
                ParseErrorSide::Right => 2,
            },
            msg = self.inner_err()
        )
    }
}
#[cfg(all(feature = "easy-functions", feature = "std"))]
impl std::error::Error for ParseErrorEither {}
#[cfg(all(
    feature = "easy-functions",
    not(feature = "std"),
    fast_tlsh_error_in_core = "stable"
))]
impl core::error::Error for ParseErrorEither {}

/// The error type describing either a generator error or an I/O error.
///
/// This type contains either:
/// *   A fuzzy hash generator error ([`GeneratorError`]) or
/// *   An I/O error ([`std::io::Error`]).
#[cfg(all(feature = "easy-functions", feature = "std"))]
#[derive(Debug)]
pub enum GeneratorOrIOError {
    /// An error caused by the fuzzy hash generator.
    GeneratorError(GeneratorError),
    /// An error caused by an internal I/O operation.
    IOError(std::io::Error),
}
#[cfg(all(feature = "easy-functions", feature = "std"))]
impl Display for GeneratorOrIOError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            GeneratorOrIOError::GeneratorError(err) => err.fmt(f),
            GeneratorOrIOError::IOError(err) => err.fmt(f),
        }
    }
}
#[cfg(all(feature = "easy-functions", feature = "std"))]
impl From<GeneratorError> for GeneratorOrIOError {
    // For wrapping with the '?' operator
    fn from(value: GeneratorError) -> Self {
        GeneratorOrIOError::GeneratorError(value)
    }
}
#[cfg(all(feature = "easy-functions", feature = "std"))]
impl From<std::io::Error> for GeneratorOrIOError {
    // For wrapping with the '?' operator
    fn from(value: std::io::Error) -> Self {
        GeneratorOrIOError::IOError(value)
    }
}
#[cfg(all(feature = "easy-functions", feature = "std"))]
impl std::error::Error for GeneratorOrIOError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GeneratorOrIOError::GeneratorError(err) => Some(err),
            GeneratorOrIOError::IOError(err) => Some(err),
        }
    }
}

mod tests;
