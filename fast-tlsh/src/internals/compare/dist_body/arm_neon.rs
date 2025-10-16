// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! NEON/ASIMD implementation (Arm) of TLSH body comparison.
//!
//! This implementation handles a 128-bit integer as 64 2-bit integers.
//!
//! On horizontal addition, an unique NEON/ASIMD feature: 64/128-bit union
//! (registers D0–D31 and Q0–Q15 are mapped to the same register space)
//! is utilized.

#![cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    any(
        all(target_arch = "aarch64", any(doc, target_feature = "neon")),
        all(
            target_arch = "arm",
            feature = "unstable",
            any(
                doc,
                all(
                    target_feature = "v7",
                    any(feature = "detect-features", target_feature = "neon")
                )
            )
        )
    )
))]
#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;
#[cfg(all(target_arch = "arm", feature = "unstable"))]
use core::arch::arm::*;

static_assertions::const_assert_eq!(super::BODY_OUTLIER_VALUE, 6);

/// Computes the distance between two 128-bit vectors and return as
/// a packed `u16` array (8 elements).
#[allow(unsafe_code)]
#[cfg_attr(
    not(all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    )),
    inline(always)
)]
#[cfg_attr(
    all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    ),
    target_feature(enable = "neon"),
    inline
)]
unsafe fn packed_distance_as_u16x8(x: uint8x16_t, y: uint8x16_t) -> uint16x8_t {
    // Constants
    let mask_dibit_01 = vreinterpretq_u32_u8(vdupq_n_u8(0b01_01_01_01));
    let mask_dibit_10 = vreinterpretq_u32_u8(vdupq_n_u8(0b10_10_10_10));
    let mask_nibble_0011 = vreinterpretq_u32_u8(vdupq_n_u8(0b0011_0011));
    let mask_byte_00001111 = vreinterpretq_u32_u8(vdupq_n_u8(0b00001111));

    let x = vreinterpretq_u32_u8(x);
    let y = vreinterpretq_u32_u8(y);
    let z = veorq_u32(x, y);

    // Step by Step evaluation (independent A and B are interleaved)
    let ta = vandq_u32(y, mask_dibit_01);
    let tb = vandq_u32(x, mask_dibit_01);
    let ta = vorrq_u32(ta, vshlq_n_u32::<1>(ta)); // * 3
    let tb = vsubq_u32(mask_dibit_10, tb);
    let ta = veorq_u32(ta, x);
    let tb = veorq_u32(tb, x);
    let sa = vandq_u32(ta, z); // SUM 1 (2-bit sliced; 0..=3)
    let tb = vandq_u32(tb, z);
    let ta = vshrq_n_u32::<2>(sa);
    let sa = vandq_u32(sa, mask_nibble_0011);
    let tb = vshrq_n_u32::<1>(tb);
    let ta = vandq_u32(ta, mask_nibble_0011);
    let tb = vorrq_u32(tb, vshlq_n_u32::<1>(tb)); // * 3
    let sa = vaddq_u32(sa, ta); // SUM 1 (4-bit sliced; 0..=6)
    let sb = vandq_u32(tb, z); // SUM 2 (2-bit sliced; 0..=3)
    let tb = vshrq_n_u32::<2>(sb);
    let sb = vandq_u32(sb, mask_nibble_0011);
    let tb = vandq_u32(tb, mask_nibble_0011);
    let sb = vaddq_u32(sb, tb); // SUM 2 (4-bit sliced; 0..=6)

    // Aggregation
    let s = vaddq_u32(sb, sa); // SUM (4-bit sliced; 0..=12)
    let t = vshrq_n_u32::<4>(s);
    let s = vandq_u32(s, mask_byte_00001111);
    let t = vandq_u32(t, mask_byte_00001111);
    let s = vaddq_u32(s, t); // SUM (8-bit sliced; 0..=24)
    vpaddlq_u8(vreinterpretq_u8_u32(s))
}

/// Computes the distance between two 32-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(
    not(all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    )),
    inline(always)
)]
#[cfg_attr(
    all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    ),
    target_feature(enable = "neon"),
    inline
)]
pub unsafe fn distance_32(body1: &[u8; 32], body2: &[u8; 32]) -> u32 {
    let px = body1 as *const u8;
    let py = body2 as *const u8;

    // First and second halfs
    let (x, y) = (vld1q_u8(px), vld1q_u8(py));
    let s1 = packed_distance_as_u16x8(x, y); // SUM (16-bit sliced; 0..=48)
    let (x, y) = (vld1q_u8(px.add(16)), vld1q_u8(py.add(16)));
    let s2 = packed_distance_as_u16x8(x, y); // SUM (16-bit sliced; 0..=48)

    // Horizontal sum
    let s = vaddq_u16(s1, s2); // Both halfs SUM (16-bit sliced; 0..=96)
    let t = vpaddlq_u16(s); // Both halfs SUM (32-bit sliced; 0..=192)
    let s = vget_high_u32(t);
    let t = vget_low_u32(t);
    let s = vadd_u32(s, t); // Both halfs SUM (32-bit sliced+reduced; 0..=384)
    vget_lane_u32::<0>(s).wrapping_add(vget_lane_u32::<1>(s))
}

/// Computes the distance between two 64-byte TLSH bodies.
#[allow(unsafe_code)]
#[cfg_attr(
    not(all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    )),
    inline(always)
)]
#[cfg_attr(
    all(
        target_arch = "arm",
        feature = "detect-features",
        feature = "unstable",
        target_feature = "v7"
    ),
    target_feature(enable = "neon"),
    inline
)]
pub unsafe fn distance_64(body1: &[u8; 64], body2: &[u8; 64]) -> u32 {
    let mut px = body1 as *const u8;
    let mut py = body2 as *const u8;

    // First and second halfs
    let mut s = vdupq_n_u16(0);
    for _ in 0..4 {
        let (x, y) = (vld1q_u8(px), vld1q_u8(py));
        // SUM (16-bit sliced; 0..=48) for each loop
        s = vaddq_u16(s, packed_distance_as_u16x8(x, y));
        (px, py) = (px.add(16), py.add(16));
    }

    // Horizontal sum
    let s = vpaddlq_u16(s); // Both halfs SUM (32-bit sliced; 0..=192)
    let t = vget_high_u32(s);
    let s = vget_low_u32(s);
    let s = vadd_u32(s, t); // Both halfs SUM (32-bit sliced+reduced; 0..=768)
    vget_lane_u32::<0>(s).wrapping_add(vget_lane_u32::<1>(s))
}
