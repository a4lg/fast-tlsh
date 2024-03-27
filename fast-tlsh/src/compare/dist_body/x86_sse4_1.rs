// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! SSE4.1 implementation (x86) of TLSH body comparison.
//!
//! This implementation handles a 128-bit integer as 64 2-bit integers.

#![cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    any(target_arch = "x86", target_arch = "x86_64"),
    any(
        feature = "detect-features",
        all(not(target_feature = "avx2"), target_feature = "sse4.1")
    )
))]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

static_assertions::const_assert_eq!(super::BODY_OUTLIER_VALUE, 6);

/// Computes the distance between two 128-bit vectors and return as
/// a packed `u32` array (4 elements).
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "sse4.1"), inline)]
unsafe fn packed_distance_as_u32x4(x: __m128i, y: __m128i) -> __m128i {
    // Constants
    let mask_dibit_01 = _mm_set1_epi8(0b01_01_01_01i8);
    let mask_dibit_10 = _mm_set1_epi8(0b10_10_10_10u8 as i8);
    let mask_nibble_0011 = _mm_set1_epi8(0b0011_0011);
    let mask_byte_00001111 = _mm_set1_epi8(0b00001111);
    let value_dword_0x01010101 = _mm_set1_epi32(0x01010101);

    let z = _mm_xor_si128(x, y);

    // Step by Step evaluation (independent A and B are interleaved)
    let ta = _mm_and_si128(y, mask_dibit_01);
    let tb = _mm_and_si128(x, mask_dibit_01);
    let ta = _mm_or_si128(ta, _mm_slli_epi32::<1>(ta)); // * 3
    let tb = _mm_sub_epi32(mask_dibit_10, tb);
    let ta = _mm_xor_si128(ta, x);
    let tb = _mm_xor_si128(tb, x);
    let sa = _mm_and_si128(ta, z); // SUM 1 (2-bit sliced; 0..=3)
    let tb = _mm_and_si128(tb, z);
    let ta = _mm_srli_epi32::<2>(sa);
    let sa = _mm_and_si128(sa, mask_nibble_0011);
    let tb = _mm_srli_epi32::<1>(tb);
    let ta = _mm_and_si128(ta, mask_nibble_0011);
    let tb = _mm_or_si128(tb, _mm_slli_epi32::<1>(tb)); // * 3
    let sa = _mm_add_epi32(sa, ta); // SUM 1 (4-bit sliced; 0..=6)
    let sb = _mm_and_si128(tb, z); // SUM 2 (2-bit sliced; 0..=3)
    let tb = _mm_srli_epi32::<2>(sb);
    let sb = _mm_and_si128(sb, mask_nibble_0011);
    let tb = _mm_and_si128(tb, mask_nibble_0011);
    let sb = _mm_add_epi32(sb, tb); // SUM 2 (4-bit sliced; 0..=6)

    // Aggregation
    let s = _mm_add_epi32(sb, sa); // SUM (4-bit sliced; 0..=12)
    let t = _mm_srli_epi32::<4>(s);
    let s = _mm_and_si128(s, mask_byte_00001111);
    let t = _mm_and_si128(t, mask_byte_00001111);
    let s = _mm_add_epi32(s, t); // SUM (8-bit sliced; 0..=24)
    let s = _mm_mullo_epi32(s, value_dword_0x01010101);
    _mm_srli_epi32::<24>(s) // SUM (32-bit sliced; 0..=96)
}

/// Computes the distance between two 32-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "sse4.1"), inline)]
pub unsafe fn distance_32(body1: &[u8; 32], body2: &[u8; 32]) -> u32 {
    let px = body1 as *const u8 as *const __m128i;
    let py = body2 as *const u8 as *const __m128i;

    // First half
    let x1 = _mm_loadu_si128(px);
    let y1 = _mm_loadu_si128(py);
    let s1 = packed_distance_as_u32x4(x1, y1); // SUM (32-bit sliced; 0..=96)

    // Second Half (just like the first half)
    let x2 = _mm_loadu_si128(px.add(1));
    let y2 = _mm_loadu_si128(py.add(1));
    let s2 = packed_distance_as_u32x4(x2, y2); // SUM (32-bit sliced; 0..=96)

    // Horizontal sum
    let s = _mm_add_epi32(s1, s2); // Both halfs SUM (32-bit sliced; 0..=192)
    let t = _mm_shuffle_epi32::<0b11_10_11_10>(s);
    let s = _mm_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=384 on lanes 0-1)
    let t = _mm_shuffle_epi32::<0b01_01_01_01>(s);
    let s = _mm_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=768 on lane 0)
    _mm_cvtsi128_si32(s) as u32
}

/// Computes the distance between two 64-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "sse2"), inline)]
pub unsafe fn distance_64(body1: &[u8; 64], body2: &[u8; 64]) -> u32 {
    let px = body1 as *const u8 as *const __m128i;
    let py = body2 as *const u8 as *const __m128i;

    let mut s = _mm_set1_epi32(0); // SUM (32-bit sliced; 0..=384) after 4 loops
    for i in 0..4 {
        let x = _mm_loadu_si128(px.add(i));
        let y = _mm_loadu_si128(py.add(i));
        s = _mm_add_epi32(s, packed_distance_as_u32x4(x, y)); // SUM (32-bit sliced; 0..=96)
    }

    // Horizontal sum
    let t = _mm_shuffle_epi32::<0b11_10_11_10>(s);
    let s = _mm_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=768 on lanes 0-1)
    let t = _mm_shuffle_epi32::<0b01_01_01_01>(s);
    let s = _mm_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=1536 on lane 0)
    _mm_cvtsi128_si32(s) as u32
}
