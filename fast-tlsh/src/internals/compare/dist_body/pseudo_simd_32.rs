// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! 32-bit Pseudo-SIMD implementation of TLSH body comparison.
//!
//! This implementation handles a 32-bit integer as 16 2-bit integers.

use core::num::Wrapping;

static_assertions::const_assert_eq!(super::BODY_OUTLIER_VALUE, 6);

/// Computes the distance between two 32-bit values (subset of TLSH bodies).
#[inline(always)]
pub(crate) fn sub_distance(x: u32, y: u32) -> u32 {
    let x = Wrapping(x);
    let y = Wrapping(y);

    // Constants
    let mask_dibit_01 = Wrapping(0x5555_5555u32);
    let mask_dibit_10 = Wrapping(0xaaaa_aaaau32);
    let mask_nibble_0011 = Wrapping(0x3333_3333u32);
    let mask_byte_00001111 = Wrapping(0x0f0f_0f0fu32);

    let z = x ^ y;

    // Step by Step evaluation
    // Independent calculation of A and B are intentionally interleaved
    // to lower dependency to the optimizer.
    let ta = y & mask_dibit_01;
    let tb = x & mask_dibit_01;
    let ta = (ta << 1) + ta; // * 3 (leave possibility of arithmetic optimization)
    let tb = mask_dibit_10 - tb;
    let ta = ta ^ x;
    let tb = tb ^ x;
    let sa = ta & z; // SUM 1 (2-bit sliced; 0..=3)
    let tb = tb & z;
    let ta = sa >> 2;
    let sa = sa & mask_nibble_0011;
    let tb = tb >> 1;
    let ta = ta & mask_nibble_0011;
    let tb = (tb << 1) + tb; // * 3 (leave possibility of arithmetic optimization)
    let sa = sa + ta; // SUM 1 (4-bit sliced; 0..=6)
    let sb = tb & z; // SUM 2 (2-bit sliced; 0..=3)
    let tb = sb >> 2;
    let sb = sb & mask_nibble_0011;
    let tb = tb & mask_nibble_0011;
    let sb = sb + tb; // SUM 2 (4-bit sliced; 0..=6)

    // Aggregation and Horizontal sum
    let s = sa + sb; // SUM (4-bit sliced; 0..=12)
    let t = s >> 4;
    let s = s & mask_byte_00001111;
    let t = t & mask_byte_00001111;
    let s = s + t; // SUM (8-bit sliced; 0..=24)
    ((s * Wrapping(0x01010101)) >> 24).0 // SUM (0..=96)
}

/// Generates distance functions like [`distance_32()`].
macro_rules! distance_func_template {
    {$($name:ident = $size:literal;)*} => {
        $(
            #[doc = concat!("Computes the distance between two ", stringify!($size), "-byte TLSH bodies.")]
            #[inline]
            pub fn $name(body1: &[u8; $size], body2: &[u8; $size]) -> u32 {
                let mut total = 0;
                for (x, y) in body1
                    .as_slice()
                    .chunks_exact(4)
                    .zip(body2.as_slice().chunks_exact(4))
                {
                    let x = u32::from_ne_bytes(x.try_into().unwrap());
                    let y = u32::from_ne_bytes(y.try_into().unwrap());
                    total += sub_distance(x, y);
                }
                total
            }
        )*
    }
}

distance_func_template! {
    distance_12 = 12;
    distance_32 = 32;
    distance_64 = 64;
}
