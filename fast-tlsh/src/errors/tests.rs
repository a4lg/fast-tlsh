// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::errors`].

#![cfg(test)]

use super::{GeneratorError, GeneratorErrorCategory, OperationError, ParseError};

#[cfg(all(feature = "easy-functions", feature = "std"))]
use super::GeneratorOrIOError;
#[cfg(feature = "easy-functions")]
use super::{ParseErrorEither, ParseErrorSide};

#[test]
fn parse_error_impls() {
    // Display
    assert_eq!(
        format!("{err}", err = ParseError::LengthIsTooLarge),
        "length field is too large"
    );
    assert_eq!(
        format!("{err}", err = ParseError::InvalidPrefix),
        "encountered an invalid prefix"
    );
    assert_eq!(
        format!("{err}", err = ParseError::InvalidCharacter),
        "encountered an invalid character"
    );
    assert_eq!(
        format!("{err}", err = ParseError::InvalidStringLength),
        "string length is invalid"
    );
    assert_eq!(
        format!("{err}", err = ParseError::InvalidChecksum),
        "has an invalid checksum field"
    );
}

#[test]
fn operation_error_impls() {
    // Display
    assert_eq!(
        format!("{err}", err = OperationError::BufferIsTooSmall),
        "buffer is too small to store the result"
    );
}

#[test]
fn generator_error_impls() {
    // Display
    assert_eq!(
        format!("{err}", err = GeneratorError::TooLargeInput),
        "input data is too large to process"
    );
    assert_eq!(
        format!("{err}", err = GeneratorError::TooSmallInput),
        "input data is too small to process"
    );
    assert_eq!(
        format!("{err}", err = GeneratorError::BucketsAreHalfEmpty),
        "approximately half or more effective buckets are empty"
    );
    assert_eq!(
        format!("{err}", err = GeneratorError::BucketsAreThreeQuarterEmpty),
        "approximately 3/4 or more effective buckets are empty"
    );
}

#[test]
fn generator_error_to_category_sizes() {
    assert_eq!(
        GeneratorError::TooLargeInput.category(),
        GeneratorErrorCategory::DataLength
    );
    assert_eq!(
        GeneratorError::TooSmallInput.category(),
        GeneratorErrorCategory::DataLength
    );
}

#[cfg(feature = "easy-functions")]
#[test]
fn parse_error_either_basic() {
    let side1 = ParseErrorSide::Left;
    let side2 = ParseErrorSide::Right;
    let inner1 = ParseError::InvalidStringLength;
    let inner2 = ParseError::LengthIsTooLarge;
    let err1 = ParseErrorEither(side1, inner1);
    let err2 = ParseErrorEither(side2, inner2);
    // Implementation: Display
    assert_eq!(
        format!("{err1}"),
        "error occurred while parsing fuzzy hash 1 (string length is invalid)"
    );
    assert_eq!(
        format!("{err2}"),
        "error occurred while parsing fuzzy hash 2 (length field is too large)"
    );
    // Decomposition
    assert_eq!(err1.side(), side1);
    assert_eq!(err2.side(), side2);
    assert_eq!(err1.inner_err(), inner1);
    assert_eq!(err2.inner_err(), inner2);
}

#[cfg(all(feature = "easy-functions", feature = "std"))]
#[test]
fn generator_or_io_error_internals() {
    use std::error::Error as _;
    use std::io::{Error, ErrorKind};

    // GeneratorError
    let orig_inner = GeneratorError::TooSmallInput;
    let err = GeneratorOrIOError::from(orig_inner);
    let inner = err
        .source()
        .unwrap()
        .downcast_ref::<GeneratorError>()
        .unwrap();
    assert_eq!(inner, &GeneratorError::TooSmallInput);
    assert_eq!(format!("{err}"), format!("{inner}"));

    // IOError
    let orig_inner = Error::from(ErrorKind::NotFound);
    let err = GeneratorOrIOError::from(orig_inner);
    let inner = err.source().unwrap().downcast_ref::<Error>().unwrap();
    assert_eq!(inner.kind(), ErrorKind::NotFound);
    assert_eq!(format!("{err}"), format!("{inner}"));
}
