// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::hash::body`].

#![cfg(test)]

use super::{FuzzyHashBody, FuzzyHashBodyData, BODY_SIZE_LONG, BODY_SIZE_NORMAL, BODY_SIZE_SHORT};

use crate::internals::compare::dist_body::naive::distance_dibits;
use crate::internals::errors::ParseError;

#[test]
fn prerequisites() {
    // Test body sizes directly with known constants.
    static_assertions::const_assert_eq!(BODY_SIZE_SHORT, 12);
    static_assertions::const_assert_eq!(BODY_SIZE_NORMAL, 32);
    static_assertions::const_assert_eq!(BODY_SIZE_LONG, 64);
}

#[test]
fn params() {
    fn test<const SIZE_BODY: usize>()
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
    {
        assert_eq!(SIZE_BODY * 4, FuzzyHashBodyData::<SIZE_BODY>::NUM_BUCKETS);
    }
    test::<BODY_SIZE_SHORT>();
    test::<BODY_SIZE_NORMAL>();
    test::<BODY_SIZE_LONG>();
}

/*
    Upper values:
        FF: 11 11 11 11
        AA: 10 10 10 10
        55: 01 01 01 01
        00: 00 00 00 00
            3<--------0
    Lower value:
        E4: 11 10 01 00
            3<--------0
*/
const HEX_U_S: &[u8] = b"FFFFFFAAAAAA555555000000";
const DATA_U_S: &[u8] = b"\xff\xff\xff\xaa\xaa\xaa\x55\x55\x55\x00\x00\x00";
const HEX_L_S: &[u8] = b"E4E4E4E4E4E4E4E4E4E4E4E4";
const DATA_L_S: &[u8] = [0xe4; 12].as_slice();
const HEX_U_M: &[u8] = b"\
    FFFFFFFFFFFFFFFFAAAAAAAAAAAAAAAA\
    55555555555555550000000000000000";
const DATA_U_M: &[u8] = b"\
    \xff\xff\xff\xff\xff\xff\xff\xff\
    \xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\
    \x55\x55\x55\x55\x55\x55\x55\x55\
    \x00\x00\x00\x00\x00\x00\x00\x00";
const HEX_L_M: &[u8] = b"\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4";
const DATA_L_M: &[u8] = [0xe4; 32].as_slice();
const HEX_U_L: &[u8] = b"\
    FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\
    AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
    55555555555555555555555555555555\
    00000000000000000000000000000000";
const DATA_U_L: &[u8] = b"\
    \xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\
    \xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\xaa\
    \x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\x55\
    \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
const HEX_L_L: &[u8] = b"\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4\
    E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4E4";
const DATA_L_L: &[u8] = [0xe4; 64].as_slice();

// TLSH's hexadecimal representation matches to the
// "plain" representation of bytes.
const HEX_RANDOM_S: &[u8] = b"801B923370C0C87B40118C7C";
const DATA_RANDOM_S: &[u8] = b"\
    \x80\x1b\x92\x33\x70\xc0\xc8\x7b\x40\x11\x8c\x7c";
const HEX_RANDOM_M: &[u8] = b"\
    CE30057ED4508B7ADFB06693009D7BA6\
    D027E02415DE610E24E1F9FEE805316F";
const DATA_RANDOM_M: &[u8] = b"\
    \xce\x30\x05\x7e\xd4\x50\x8b\x7a\xdf\xb0\x66\x93\x00\x9d\x7b\xa6\
    \xd0\x27\xe0\x24\x15\xde\x61\x0e\x24\xe1\xf9\xfe\xe8\x05\x31\x6f";
const HEX_RANDOM_L: &[u8] = b"\
    8B0DBA1D693E524CFC416D44A73BE0C4\
    AA3D5F5E90BA979B530EE30B1528FC9B\
    47055A91C2086A0292FB2C1F6B05936D\
    F3FA6222845C9FA740D8E3A6B42986E8";
const DATA_RANDOM_L: &[u8] = b"\
    \x8b\x0d\xba\x1d\x69\x3e\x52\x4c\xfc\x41\x6d\x44\xa7\x3b\xe0\xc4\
    \xaa\x3d\x5f\x5e\x90\xba\x97\x9b\x53\x0e\xe3\x0b\x15\x28\xfc\x9b\
    \x47\x05\x5a\x91\xc2\x08\x6a\x02\x92\xfb\x2c\x1f\x6b\x05\x93\x6d\
    \xf3\xfa\x62\x22\x84\x5c\x9f\xa7\x40\xd8\xe3\xa6\xb4\x29\x86\xe8";

#[test]
fn from_raw_ordering() {
    fn test<const SIZE_BODY: usize>(data_u: &[u8], data_l: &[u8])
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
    {
        let body = FuzzyHashBodyData::<SIZE_BODY>::from_raw(
            core::convert::TryInto::<[u8; SIZE_BODY]>::try_into(data_u).unwrap(),
        );
        for index in 0..SIZE_BODY {
            let q = body.quartile(index);
            // Low value comes first.
            let expected = (index / SIZE_BODY) as u8;
            assert_eq!(q, expected);
        }
        let body = FuzzyHashBodyData::<SIZE_BODY>::from_raw(
            core::convert::TryInto::<[u8; SIZE_BODY]>::try_into(data_l).unwrap(),
        );
        for index in 0..SIZE_BODY {
            let q = body.quartile(index);
            // Low value in the lower bits of each byte.
            let expected = (index % 4) as u8;
            assert_eq!(q, expected);
        }
    }
    test::<BODY_SIZE_SHORT>(DATA_U_S, DATA_L_S);
    test::<BODY_SIZE_NORMAL>(DATA_U_M, DATA_L_M);
    test::<BODY_SIZE_LONG>(DATA_U_L, DATA_L_L);
}

#[test]
fn from_str_bytes_equality() {
    fn test<const SIZE_BODY: usize>(input: &[u8], input_hex: &[u8])
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
    {
        let body1 = FuzzyHashBodyData::<SIZE_BODY>::from_raw(
            core::convert::TryInto::<[u8; SIZE_BODY]>::try_into(input).unwrap(),
        );
        let body2 = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(input_hex).unwrap();
        assert_eq!(body1, body2);
    }
    test::<BODY_SIZE_SHORT>(DATA_U_S, HEX_U_S);
    test::<BODY_SIZE_SHORT>(DATA_L_S, HEX_L_S);
    test::<BODY_SIZE_SHORT>(DATA_RANDOM_S, HEX_RANDOM_S);
    test::<BODY_SIZE_NORMAL>(DATA_U_M, HEX_U_M);
    test::<BODY_SIZE_NORMAL>(DATA_L_M, HEX_L_M);
    test::<BODY_SIZE_NORMAL>(DATA_RANDOM_M, HEX_RANDOM_M);
    test::<BODY_SIZE_LONG>(DATA_U_L, HEX_U_L);
    test::<BODY_SIZE_LONG>(DATA_L_L, HEX_L_L);
    test::<BODY_SIZE_LONG>(DATA_RANDOM_L, HEX_RANDOM_L);
}

#[test]
fn from_str_bytes_errors() {
    fn test<const SIZE_BODY: usize>() {
        let buffer = "aa".repeat(SIZE_BODY - 1); // insufficient size
        let result = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(buffer.as_bytes());
        assert_eq!(result, Err(ParseError::InvalidStringLength));
        let buffer = "aa".repeat(SIZE_BODY + 1); // excess size
        let result = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(buffer.as_bytes());
        assert_eq!(result, Err(ParseError::InvalidStringLength));
        let buffer = "@@".repeat(SIZE_BODY); // with invalid character
        let result = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(buffer.as_bytes());
        assert_eq!(result, Err(ParseError::InvalidCharacter));
        let buffer = "aa".repeat(SIZE_BODY); // with invalid character
        let result = FuzzyHashBodyData::<SIZE_BODY>::from_str_bytes(buffer.as_bytes());
        assert!(result.is_ok());
    }
    test::<BODY_SIZE_SHORT>();
    test::<BODY_SIZE_NORMAL>();
    test::<BODY_SIZE_LONG>();
}

#[test]
fn compare_dibits_single() {
    fn test<const SIZE_BODY: usize>()
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
    {
        for index in 0..FuzzyHashBodyData::<SIZE_BODY>::NUM_BUCKETS {
            for a in 0..4 {
                let mut body_a = FuzzyHashBodyData::<SIZE_BODY>::from_raw([0; SIZE_BODY]);
                body_a.data[SIZE_BODY - 1 - index / 4] |= a << (2 * (index % 4));
                let body_a = body_a;
                for b in 0..4 {
                    let mut body_b = FuzzyHashBodyData::<SIZE_BODY>::from_raw([0; SIZE_BODY]);
                    body_b.data[SIZE_BODY - 1 - index / 4] |= b << (2 * (index % 4));
                    let body_b = body_b;
                    let expected = distance_dibits(a, b);
                    assert_eq!(body_a.compare(&body_b), expected);
                }
            }
        }
    }
    test::<BODY_SIZE_SHORT>();
    test::<BODY_SIZE_NORMAL>();
    test::<BODY_SIZE_LONG>();
}

#[test]
fn compare_dibits_all() {
    fn test<const SIZE_BODY: usize>()
    where
        FuzzyHashBodyData<SIZE_BODY>: FuzzyHashBody,
    {
        for a in 0..4 {
            let value_a = (0..4).fold(0u8, |x, _| (x << 2) | a);
            let body_a = FuzzyHashBodyData::from_raw([value_a; SIZE_BODY]);
            for b in 0..4 {
                let value_b = (0..4).fold(0u8, |x, _| (x << 2) | b);
                let body_b = FuzzyHashBodyData::from_raw([value_b; SIZE_BODY]);
                let expected =
                    distance_dibits(a, b) * FuzzyHashBodyData::<SIZE_BODY>::NUM_BUCKETS as u32;
                assert_eq!(body_a.compare(&body_b), expected);
            }
        }
    }
    test::<BODY_SIZE_SHORT>();
    test::<BODY_SIZE_NORMAL>();
    test::<BODY_SIZE_LONG>();
}
