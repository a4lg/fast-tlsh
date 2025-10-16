// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! AVX2 implementation (x86) of TLSH bucket aggregation.
//!
//! This implementation handles 8 buckets at once.

#![cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-bucket-aggregation",
    any(target_arch = "x86", target_arch = "x86_64"),
    any(feature = "detect-features", target_feature = "avx2")
))]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Aggregate 8 buckets into the 2-byte sub-digest based on three quartiles.
///
/// It is assumed to be:
/// *   `q1 <= q2`
/// *   `q2 <= q3`
#[allow(unsafe_code)]
#[cfg_attr(not(feature = "detect-features"), inline(always))]
#[cfg_attr(feature = "detect-features", target_feature(enable = "avx2"), inline)]
unsafe fn sub_aggregation(buckets: &[u32], q1: u32, q2: u32, q3: u32) -> (u8, u8) {
    assert!(buckets.len() >= 8);
    let qv1 = _mm256_set1_epi32((q1 ^ 0x80000000) as i32);
    let qv2 = _mm256_set1_epi32((q2 ^ 0x80000000) as i32);
    let qv3 = _mm256_set1_epi32((q3 ^ 0x80000000) as i32);
    let hibit = _mm256_set1_epi32(0x80000000u32 as i32);
    // [-128, -128, -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0] * 2
    let shufb_lo = _mm256_set_epi8(
        -128, -128, -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0, -128,
        -128, -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0,
    );
    // [-128, -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0, -128] * 2
    let shufb_hi = _mm256_set_epi8(
        -128, -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0, -128, -128,
        -128, -128, -128, -128, -128, -128, -128, 12, -128, 8, -128, 4, -128, 0, -128,
    );
    let data = _mm256_xor_si256(
        _mm256_loadu_si256(buckets.as_ptr() as *const __m256i),
        hibit,
    );
    let qc2 = _mm256_cmpgt_epi32(data, qv2);
    let qb1 = _mm256_shuffle_epi8(qc2, shufb_hi);
    let qb1 = _mm256_movemask_epi8(qb1) as u32;
    let qc1 = _mm256_cmpgt_epi32(data, qv1);
    let qc3 = _mm256_cmpgt_epi32(data, qv3);
    let qb0 = _mm256_xor_si256(qc2, qc1);
    let qb0 = _mm256_xor_si256(qb0, qc3);
    let qb0 = _mm256_shuffle_epi8(qb0, shufb_lo);
    let qb0 = _mm256_movemask_epi8(qb0) as u32;
    let qb = qb1 | qb0;
    ((qb >> 16) as u8, qb as u8)
}

/// Generates aggregation functions like [`aggregate_128()`].
macro_rules! aggregation_func_template {
    {$($name:ident = ($size_small:literal, $size_large:literal);)*} => {
        $(
            #[doc = concat!(
                "Aggregate ",
                stringify!($size_large),
                " buckets into the ",
                stringify!($size_small),
                "-byte digest based on three quartiles.\n",
                "\n",
                "This function requires that:\n",
                "*   `q1 <= q2`\n",
                "*   `q2 <= q3`"
            )]
            #[allow(unsafe_code)]
            #[cfg_attr(not(feature = "detect-features"), inline(always))]
            #[cfg_attr(feature = "detect-features", target_feature(enable = "avx2"), inline)]
            pub(super) unsafe fn $name(
                out: &mut [u8; $size_small],
                buckets: &[u32; $size_large],
                q1: u32,
                q2: u32,
                q3: u32
            ) {
                for (out, subbuckets) in out
                    .chunks_mut(2)
                    .rev()
                    .zip(buckets.as_slice().chunks_exact(8))
                {
                    (out[0], out[1]) = sub_aggregation(subbuckets, q1, q2, q3);
                }
            }
        )*
    }
}

aggregation_func_template! {
    aggregate_48  = (12,  48);
    aggregate_128 = (32, 128);
    aggregate_256 = (64, 256);
}
