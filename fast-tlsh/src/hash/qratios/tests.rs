// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::hash::qratios`].

#![cfg(test)]

use super::FuzzyHashQRatios;

use crate::internals::errors::ParseError;

#[test]
fn qratio_pair() {
    for q2 in 0..16u8 {
        for q1 in 0..16u8 {
            let qratios = FuzzyHashQRatios::new(q1, q2);
            assert_eq!(qratios.q1ratio(), q1);
            assert_eq!(qratios.q2ratio(), q2);
        }
    }
}

#[test]
fn qratios_encoding() {
    for value in u8::MIN..=u8::MAX {
        let qratios = FuzzyHashQRatios::from_raw(value);
        // Encoding: low 4 bits is Q1 ratio.
        let q1 = value & 0x0f;
        let q2 = (value >> 4) & 0x0f;
        assert_eq!(qratios.value(), value);
        assert_eq!(qratios.q1ratio(), q1);
        assert_eq!(qratios.q2ratio(), q2);
    }
}

#[test]
fn qratio_from_str_bytes_fail_len() {
    const ZEROS: &[u8] = &[b'0'; 3];
    // Length 1: invalid length
    assert_eq!(
        FuzzyHashQRatios::from_str_bytes(&ZEROS[0..1]),
        Err(ParseError::InvalidStringLength)
    );
    // Length 2: parser runs
    assert_eq!(
        FuzzyHashQRatios::from_str_bytes(&ZEROS[0..2]),
        Ok(FuzzyHashQRatios::from_raw(0))
    );
    // Length 3: invalid length
    assert_eq!(
        FuzzyHashQRatios::from_str_bytes(&ZEROS[0..3]),
        Err(ParseError::InvalidStringLength)
    );
}

#[test]
fn qratio_from_str_bytes_endianness() {
    for value in u8::MIN..=u8::MAX {
        let s: String = format!("{value:02X}").chars().rev().collect();
        assert_eq!(
            FuzzyHashQRatios::from_str_bytes(s.as_bytes()),
            Ok(FuzzyHashQRatios::from_raw(value))
        );
    }
}

#[test]
#[should_panic]
fn qratios_init_fail_q1() {
    // 16 is an invalid value for Q ratio.
    let _qratios = FuzzyHashQRatios::new(16, 0);
}

#[test]
#[should_panic]
fn qratios_init_fail_q2() {
    // 16 is an invalid value for Q ratio.
    let _qratios = FuzzyHashQRatios::new(0, 16);
}
