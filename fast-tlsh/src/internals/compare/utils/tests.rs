// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: `crate::internals::compare::utils`.

#![cfg(test)]

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::internals::compare::utils::generic;

#[test]
fn mod_diff_16() {
    for y in 0..16u8 {
        for x in 0..16u8 {
            // Constant implementation matches with the generic implementation.
            assert_eq!(
                super::distance_on_ring_mod(x, y, 16),
                generic::distance_on_ring_mod(x, y, 16)
            );
            // The function is symmetric.
            assert_eq!(
                super::distance_on_ring_mod(x, y, 16),
                super::distance_on_ring_mod(y, x, 16)
            );
            // mod_diff(x, y, 16) in u8 works as expected.
            assert_eq!(
                generic::distance_on_ring_mod(x, y, 16) as u16,
                generic::distance_on_ring_mod(x as u16, y as u16, 16)
            );
        }
    }
}

#[test]
fn mod_diff_16_dependency() {
    let mut values = HashMap::<u8, u8>::new();
    for y in 0..16u8 {
        for x in 0..16u8 {
            // mod_diff(x, y, 16) only depends on the wrapped difference % 16.
            let diff = x.wrapping_sub(y) % 16;
            let dist = super::distance_on_ring_mod(x, y, 16);
            match values.entry(diff) {
                Entry::Occupied(e) => {
                    assert_eq!(*(e.get()), dist);
                }
                Entry::Vacant(e) => {
                    e.insert(dist);
                }
            }
        }
    }
}

#[test]
fn mod_diff_16_max() {
    assert_eq!(
        (0..16u8)
            .map(|x| super::distance_on_ring_mod(x, 0, 16))
            .max(),
        Some(0x08),
    );
}

#[test]
fn mod_diff_256() {
    for y in u8::MIN..=u8::MAX {
        for x in u8::MIN..=u8::MAX {
            // Constant implementation matches with the generic implementation.
            assert_eq!(
                super::distance_on_ring_mod(x, y, 0),
                generic::distance_on_ring_mod(x, y, 0)
            );
            // The function is symmetric.
            assert_eq!(
                super::distance_on_ring_mod(x, y, 0),
                super::distance_on_ring_mod(y, x, 0)
            );
            // mod_diff(x as u16, y as u16, 256) (where x and y u8)
            // equals distance_on_ring_mod::<u8>(x, y, 0).
            assert_eq!(
                generic::distance_on_ring_mod(x, y, 0) as u16,
                generic::distance_on_ring_mod(x as u16, y as u16, 256)
            );
        }
    }
}

#[test]
fn mod_diff_256_dependency() {
    let mut values = HashMap::<u8, u8>::new();
    for y in u8::MIN..=u8::MAX {
        for x in u8::MIN..=u8::MAX {
            // mod_diff(x, y, 256) only depends on the wrapped difference.
            let diff = x.wrapping_sub(y);
            let dist = super::distance_on_ring_mod(x, y, 0);
            match values.entry(diff) {
                Entry::Occupied(e) => {
                    assert_eq!(*(e.get()), dist);
                }
                Entry::Vacant(e) => {
                    e.insert(dist);
                }
            }
        }
    }
}

#[test]
fn mod_diff_256_max() {
    assert_eq!(
        (u8::MIN..=u8::MAX)
            .map(|x| super::distance_on_ring_mod(x, 0, 0))
            .max(),
        Some(0x80),
    );
}
