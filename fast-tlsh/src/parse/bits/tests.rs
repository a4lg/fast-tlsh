// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::parse::bits`].

#![cfg(test)]

use super::{naive, swap_nibble_in_u8};

#[test]
fn examples() {
    assert_eq!(swap_nibble_in_u8(0x12), 0x21);
    assert_eq!(swap_nibble_in_u8(0x7c), 0xc7);
    assert_eq!(swap_nibble_in_u8(0x88), 0x88);
}

#[test]
fn exhaustive() {
    for value in u8::MIN..=u8::MAX {
        // 0x12 -> "21", equivalent to swapping nibbles inside a byte.
        let s: String = format!("{value:02x}").chars().rev().collect();
        assert_eq!(
            u8::from_str_radix(s.as_str(), 16),
            Ok(swap_nibble_in_u8(value))
        );
    }
}

#[test]
fn equivalence_optimized_impl() {
    for value in u8::MIN..=u8::MAX {
        assert_eq!(
            super::swap_nibble_in_u8(value),
            naive::swap_nibble_in_u8(value)
        );
    }
}
