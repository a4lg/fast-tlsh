// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! AVX2 implementation (x86) of TLSH body comparison.
//!
//! This implementation handles a 256-bit integer as 128 2-bit integers.

#![cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    any(target_arch = "x86", target_arch = "x86_64"),
    any(feature = "detect-features", target_feature = "avx2")
))]
#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

static_assertions::const_assert_eq!(super::BODY_OUTLIER_VALUE, 6);

/// Computes the distance between two 256-bit vectors and return as
/// a packed `u32` array (8 elements).
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "avx2"), inline)]
unsafe fn packed_distance_as_u32x8(x: __m256i, y: __m256i) -> __m256i {
    // Constants
    let mask_dibit_01 = _mm256_set1_epi8(0b01_01_01_01i8);
    let mask_dibit_10 = _mm256_set1_epi8(0b10_10_10_10u8 as i8);
    let mask_nibble_0011 = _mm256_set1_epi8(0b0011_0011);
    let mask_byte_00001111 = _mm256_set1_epi8(0b00001111);
    let value_dword_0x01010101 = _mm256_set1_epi32(0x01010101);

    let z = _mm256_xor_si256(x, y);

    // Step by Step evaluation (independent A and B are interleaved)
    let ta = _mm256_and_si256(y, mask_dibit_01);
    let tb = _mm256_and_si256(x, mask_dibit_01);
    let ta = _mm256_or_si256(ta, _mm256_slli_epi32::<1>(ta)); // * 3
    let tb = _mm256_sub_epi32(mask_dibit_10, tb);
    let ta = _mm256_xor_si256(ta, x);
    let tb = _mm256_xor_si256(tb, x);
    let sa = _mm256_and_si256(ta, z); // SUM 1 (2-bit sliced; 0..=3)
    let tb = _mm256_and_si256(tb, z);
    let ta = _mm256_srli_epi32::<2>(sa);
    let sa = _mm256_and_si256(sa, mask_nibble_0011);
    let tb = _mm256_srli_epi32::<1>(tb);
    let ta = _mm256_and_si256(ta, mask_nibble_0011);
    let tb = _mm256_or_si256(tb, _mm256_slli_epi32::<1>(tb)); // * 3
    let sa = _mm256_add_epi32(sa, ta); // SUM 1 (4-bit sliced; 0..=6)
    let sb = _mm256_and_si256(tb, z); // SUM 2 (2-bit sliced; 0..=3)
    let tb = _mm256_srli_epi32::<2>(sb);
    let sb = _mm256_and_si256(sb, mask_nibble_0011);
    let tb = _mm256_and_si256(tb, mask_nibble_0011);
    let sb = _mm256_add_epi32(sb, tb); // SUM 2 (4-bit sliced; 0..=6)

    // Aggregation
    let s = _mm256_add_epi32(sb, sa); // SUM (4-bit sliced; 0..=12)
    let t = _mm256_srli_epi32::<4>(s);
    let s = _mm256_and_si256(s, mask_byte_00001111);
    let t = _mm256_and_si256(t, mask_byte_00001111);
    let s = _mm256_add_epi32(s, t); // SUM (8-bit sliced; 0..=24)
    let s = _mm256_mullo_epi32(s, value_dword_0x01010101);
    _mm256_srli_epi32::<24>(s) // SUM (32-bit sliced; 0..=96)
}

/// Computes the distance between two 32-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "avx2"), inline)]
pub unsafe fn distance_32(body1: &[u8; 32], body2: &[u8; 32]) -> u32 {
    let x = _mm256_loadu_si256(body1 as *const u8 as *const __m256i);
    let y = _mm256_loadu_si256(body2 as *const u8 as *const __m256i);
    let s = packed_distance_as_u32x8(x, y);

    // Horizontal sum
    let t = _mm256_shuffle_epi32::<0b11_10_11_10>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=192 on lanes 0,1,4,5)
    let t = _mm256_shuffle_epi32::<0b01_01_01_01>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=384 on lanes 0,4)
    let s0 = _mm256_extract_epi32::<0>(s) as u32;
    let s1 = _mm256_extract_epi32::<4>(s) as u32;
    s0 + s1
}

/// Computes the distance between two 64-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "avx2"), inline)]
pub unsafe fn distance_64(body1: &[u8; 64], body2: &[u8; 64]) -> u32 {
    let px = body1 as *const u8 as *const __m256i;
    let py = body2 as *const u8 as *const __m256i;

    let x = _mm256_loadu_si256(px);
    let y = _mm256_loadu_si256(py);
    let s = packed_distance_as_u32x8(x, y);
    // Horizontal sum
    let t = _mm256_shuffle_epi32::<0b11_10_11_10>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=192 on lanes 0,1,4,5)
    let t = _mm256_shuffle_epi32::<0b01_01_01_01>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=384 on lanes 0,4)
    let s0 = _mm256_extract_epi32::<0>(s) as u32;
    let s1 = _mm256_extract_epi32::<4>(s) as u32;
    let v0 = s0 + s1;

    let x = _mm256_loadu_si256(px.add(1));
    let y = _mm256_loadu_si256(py.add(1));
    let s = packed_distance_as_u32x8(x, y);
    // Horizontal sum
    let t = _mm256_shuffle_epi32::<0b11_10_11_10>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=192 on lanes 0,1,4,5)
    let t = _mm256_shuffle_epi32::<0b01_01_01_01>(s);
    let s = _mm256_add_epi32(s, t); // Both halfs SUM (32-bit sliced; 0..=384 on lanes 0,4)
    let s0 = _mm256_extract_epi32::<0>(s) as u32;
    let s1 = _mm256_extract_epi32::<4>(s) as u32;
    let v1 = s0 + s1;

    v0 + v1
}
