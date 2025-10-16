// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! WebAssembly SIMD implementation of TLSH bucket aggregation.
//!
//! This implementation handles 4 buckets at once.

#![cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-bucket-aggregation",
    target_arch = "wasm32",
    any(doc, target_feature = "simd128")
))]
#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

/// Aggregate 4 buckets into the 1-byte sub-digest based on three quartiles.
///
/// It is assumed to be:
/// *   `q1 <= q2`
/// *   `q2 <= q3`
#[allow(unsafe_code)]
#[inline(always)]
unsafe fn sub_aggregation(buckets: &[u32], q1: u32, q2: u32, q3: u32) -> u8 {
    assert!(buckets.len() >= 4);
    let qv1 = u32x4_splat(q1);
    let qv2 = u32x4_splat(q2);
    let qv3 = u32x4_splat(q3);
    let data = v128_load(buckets.as_ptr() as *const v128);
    let qc2 = u32x4_gt(data, qv2);
    let qb1 = qc2;
    let qb1 = u16x8_bitmask(qb1) & 0xaa;
    let qc1 = u32x4_gt(data, qv1);
    let qc3 = u32x4_gt(data, qv3);
    let qb0 = v128_xor(qc2, qc1);
    let qb0 = v128_xor(qb0, qc3);
    let qb0 = u16x8_bitmask(qb0) & 0x55;
    qb0 | qb1
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
            #[inline]
            pub(super) unsafe fn $name(
                out: &mut [u8; $size_small],
                buckets: &[u32; $size_large],
                q1: u32,
                q2: u32,
                q3: u32
            ) {
                for (out, subbuckets) in out.iter_mut().rev().zip(buckets.as_slice().chunks_exact(4)) {
                    *out = sub_aggregation(subbuckets, q1, q2, q3);
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
