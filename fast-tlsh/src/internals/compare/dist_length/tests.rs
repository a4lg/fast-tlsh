// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::compare::dist_length`].

#![cfg(test)]

use super::naive;

#[cfg(feature = "opt-dist-length-table")]
use super::LengthDistanceTableType;

#[test]
fn arithmetic_correctness_naive() {
    // No arithmetic overflow occurs on the naïve implementation.
    // 0x80 is the maximum value of mod_diff(x, y, 256).
    assert!(0x80u32.checked_mul(length_mult!()).is_some());
}

#[cfg(feature = "opt-dist-length-table")]
#[test]
fn table_consistency()
where
    LengthDistanceTableType: From<u8>,
    u32: From<LengthDistanceTableType>,
{
    // Above constraints make sures that u8 ⊆ LengthDistanceTableType ⊆ u32.
    // 0x80 is the maximum value of mod_diff(x, y, 256).
    let dist = LengthDistanceTableType::from(0x80u8);
    assert!(dist.checked_mul(length_mult!()).is_some());
}

#[test]
fn equivalence_optimized_impl() {
    for lvalue2 in u8::MIN..=u8::MAX {
        for lvalue1 in u8::MIN..=u8::MAX {
            assert_eq!(
                super::distance(lvalue1, lvalue2),
                naive::distance(lvalue1, lvalue2)
            );
        }
    }
}
