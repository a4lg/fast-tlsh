// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::compare::dist_body`].

#![cfg(test)]

use super::naive::{self, distance_dibits};
use super::{pseudo_simd_32, pseudo_simd_64};

use crate::hash::body::{BODY_SIZE_LONG, BODY_SIZE_NORMAL, BODY_SIZE_SHORT};

#[test]
fn max_quartile_distance() {
    // Before substitution by BODY_OUTLIAR_VALUE, the maximum value is 3.
    for x in 0b00..=0b11u8 {
        for y in 0b00..=0b11u8 {
            assert!(x.abs_diff(y) <= 0b11);
        }
    }
}

trait BodyDistanceImpls<const SIZE_BODY: usize> {
    fn naive(body1: &[u8; SIZE_BODY], body2: &[u8; SIZE_BODY]) -> u32;
    fn fast(body1: &[u8; SIZE_BODY], body2: &[u8; SIZE_BODY]) -> u32;
    fn pseudo_simd_32(body1: &[u8; SIZE_BODY], body2: &[u8; SIZE_BODY]) -> u32;
    fn pseudo_simd_64(body1: &[u8; SIZE_BODY], body2: &[u8; SIZE_BODY]) -> u32;
}

struct BodyDistance<const SIZE_BODY: usize>;

impl BodyDistanceImpls<BODY_SIZE_SHORT> for BodyDistance<BODY_SIZE_SHORT> {
    fn naive(body1: &[u8; BODY_SIZE_SHORT], body2: &[u8; BODY_SIZE_SHORT]) -> u32 {
        naive::distance(body1, body2)
    }

    fn fast(body1: &[u8; BODY_SIZE_SHORT], body2: &[u8; BODY_SIZE_SHORT]) -> u32 {
        super::distance_12(body1, body2)
    }

    fn pseudo_simd_32(body1: &[u8; BODY_SIZE_SHORT], body2: &[u8; BODY_SIZE_SHORT]) -> u32 {
        pseudo_simd_32::distance_12(body1, body2)
    }

    fn pseudo_simd_64(body1: &[u8; BODY_SIZE_SHORT], body2: &[u8; BODY_SIZE_SHORT]) -> u32 {
        // THIS IS INTENTIONAL (since there's no distance_12 on pseudo_simd_64)
        pseudo_simd_32::distance_12(body1, body2)
    }
}

impl BodyDistanceImpls<BODY_SIZE_NORMAL> for BodyDistance<BODY_SIZE_NORMAL> {
    fn naive(body1: &[u8; BODY_SIZE_NORMAL], body2: &[u8; BODY_SIZE_NORMAL]) -> u32 {
        naive::distance(body1, body2)
    }

    fn fast(body1: &[u8; BODY_SIZE_NORMAL], body2: &[u8; BODY_SIZE_NORMAL]) -> u32 {
        super::distance_32(body1, body2)
    }

    fn pseudo_simd_32(body1: &[u8; BODY_SIZE_NORMAL], body2: &[u8; BODY_SIZE_NORMAL]) -> u32 {
        pseudo_simd_32::distance_32(body1, body2)
    }

    fn pseudo_simd_64(body1: &[u8; BODY_SIZE_NORMAL], body2: &[u8; BODY_SIZE_NORMAL]) -> u32 {
        pseudo_simd_64::distance_32(body1, body2)
    }
}

impl BodyDistanceImpls<BODY_SIZE_LONG> for BodyDistance<BODY_SIZE_LONG> {
    fn naive(body1: &[u8; BODY_SIZE_LONG], body2: &[u8; BODY_SIZE_LONG]) -> u32 {
        naive::distance(body1, body2)
    }

    fn fast(body1: &[u8; BODY_SIZE_LONG], body2: &[u8; BODY_SIZE_LONG]) -> u32 {
        super::distance_64(body1, body2)
    }

    fn pseudo_simd_32(body1: &[u8; BODY_SIZE_LONG], body2: &[u8; BODY_SIZE_LONG]) -> u32 {
        pseudo_simd_32::distance_64(body1, body2)
    }

    fn pseudo_simd_64(body1: &[u8; BODY_SIZE_LONG], body2: &[u8; BODY_SIZE_LONG]) -> u32 {
        pseudo_simd_64::distance_64(body1, body2)
    }
}

#[test]
fn equivalence_optimized_impl() {
    fn test<const SIZE_BODY: usize>()
    where
        BodyDistance<SIZE_BODY>: BodyDistanceImpls<SIZE_BODY>,
    {
        // Single dibit difference
        for index in 0..SIZE_BODY * 4 {
            for a in 0..4 {
                let mut body_a = [0u8; SIZE_BODY];
                body_a[SIZE_BODY - 1 - index / 4] |= a << (2 * (index % 4));
                let body_a = body_a;
                for b in 0..4 {
                    let mut body_b = [0u8; SIZE_BODY];
                    body_b[SIZE_BODY - 1 - index / 4] |= b << (2 * (index % 4));
                    let body_b = body_b;
                    let expected = distance_dibits(a, b);
                    assert_eq!(BodyDistance::<SIZE_BODY>::naive(&body_a, &body_b), expected);
                    assert_eq!(BodyDistance::<SIZE_BODY>::fast(&body_a, &body_b), expected);
                    assert_eq!(
                        BodyDistance::<SIZE_BODY>::pseudo_simd_32(&body_a, &body_b),
                        expected
                    );
                    assert_eq!(
                        BodyDistance::<SIZE_BODY>::pseudo_simd_64(&body_a, &body_b),
                        expected
                    );
                }
            }
        }
        // All dibit difference
        for a in 0..4 {
            let value_a = (0..4).fold(0u8, |x, _| (x << 2) | a);
            let body_a = [value_a; SIZE_BODY];
            for b in 0..4 {
                let value_b = (0..4).fold(0u8, |x, _| (x << 2) | b);
                let body_b = [value_b; SIZE_BODY];
                let expected = distance_dibits(a, b) * (SIZE_BODY * 4) as u32;
                assert_eq!(BodyDistance::<SIZE_BODY>::naive(&body_a, &body_b), expected);
                assert_eq!(BodyDistance::<SIZE_BODY>::fast(&body_a, &body_b), expected);
                assert_eq!(
                    BodyDistance::<SIZE_BODY>::pseudo_simd_32(&body_a, &body_b),
                    expected
                );
                assert_eq!(
                    BodyDistance::<SIZE_BODY>::pseudo_simd_64(&body_a, &body_b),
                    expected
                );
            }
        }
    }
    test::<BODY_SIZE_SHORT>();
    test::<BODY_SIZE_NORMAL>();
    test::<BODY_SIZE_LONG>();
}
