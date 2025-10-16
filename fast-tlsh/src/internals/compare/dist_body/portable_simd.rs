// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Portable SIMD implementation (Nightly Rust) of TLSH body comparison.
//!
//! This implementation handles each body as a single SIMD variable
//! (either 256-bits or 512-bits).

#![cfg(all(feature = "simd-portable", feature = "opt-simd-body-comparison"))]

use core::simd::num::SimdUint;
use core::simd::{LaneCount, Simd, SupportedLaneCount, ToBytes};

static_assertions::const_assert_eq!(super::BODY_OUTLIER_VALUE, 6);

/// Computes the distance between two `N_8`-byte TLSH bodies.
///
/// It uses [`u32`] SIMD computation for the final sum and only supports where
/// `N_8 % 4 == 0` (and the lane count `N_8` is supported by Rust).
#[inline(always)]
fn distance<const N_8: usize, const N_32: usize>(body1: &[u8; N_8], body2: &[u8; N_8]) -> u32
where
    LaneCount<N_8>: SupportedLaneCount,
    LaneCount<N_32>: SupportedLaneCount,
    Simd<u32, N_32>: ToBytes<Bytes = Simd<u8, N_8>>,
{
    let x = Simd::<u32, N_32>::from_ne_bytes(Simd::<u8, N_8>::from_array(*body1));
    let y = Simd::<u32, N_32>::from_ne_bytes(Simd::<u8, N_8>::from_array(*body2));
    let z = x ^ y;

    // Constants
    let mask_dibit_01 = Simd::<u32, N_32>::splat(0x55555555);
    let mask_dibit_10 = Simd::<u32, N_32>::splat(0xaaaaaaaa);
    let mask_nibble_0011 = Simd::<u32, N_32>::splat(0x33333333);
    let mask_byte_00001111 = Simd::<u32, N_32>::splat(0x0f0f0f0f);
    let value_dword_0x01010101 = Simd::<u32, N_32>::splat(0x01010101);

    // Step by Step evaluation (independent A and B are interleaved)
    let ta = y & mask_dibit_01;
    let tb = x & mask_dibit_01;
    let ta = ta | (ta << 1); // * 3
    let tb = mask_dibit_10 - tb;
    let ta = ta ^ x;
    let tb = tb ^ x;
    let sa = ta & z; // SUM 1 (2-bit sliced; 0..=3)
    let tb = tb & z;
    let ta = sa >> 2;
    let sa = sa & mask_nibble_0011;
    let tb = tb >> 1;
    let ta = ta & mask_nibble_0011;
    let tb = tb | (tb << 1); // * 3
    let sa = sa + ta; // SUM 1 (4-bit sliced; 0..=6)
    let sb = tb & z; // SUM 2 (2-bit sliced; 0..=3)
    let tb = sb >> 2;
    let sb = sb & mask_nibble_0011;
    let tb = tb & mask_nibble_0011;
    let sb = sb + tb; // SUM 2 (4-bit sliced; 0..=6)

    // Aggregation
    let s = sb + sa; // SUM (4-bit sliced; 0..=12)
    let t = s >> 4;
    let s = s & mask_byte_00001111;
    let t = t & mask_byte_00001111;
    let s = s + t; // SUM (8-bit sliced; 0..=24)
    let s = (s * value_dword_0x01010101) >> 24; // SUM (32-bit sliced; 0..=96)
    s.reduce_sum()
}

/// Computes the distance between two 32-byte TLSH bodies.
#[inline]
pub fn distance_32(body1: &[u8; 32], body2: &[u8; 32]) -> u32 {
    distance::<32, 8>(body1, body2)
}

/// Computes the distance between two 64-byte TLSH bodies.
#[inline]
pub fn distance_64(body1: &[u8; 64], body2: &[u8; 64]) -> u32 {
    distance::<64, 16>(body1, body2)
}
