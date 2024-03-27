// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Portable SIMD implementation (Nightly Rust) of TLSH bucket aggregation.
//!
//! This implementation handles up to 64 buckets at once.

#![cfg(all(feature = "simd-portable", feature = "opt-simd-bucket-aggregation"))]

use core::simd::cmp::SimdPartialOrd;
use core::simd::{LaneCount, Simd, SupportedLaneCount};

/// The trait to represent valid correspondence between number of partial bucket
/// entries, intermediate data and the output.
///
/// *   `N_BYTES`: The number of bytes in the output.
/// *   `N_HALF_BYTES`: The half of `N_HALF_BYTES`
///     (for intermediate byte-based handling).
/// *   `N_ELEMS`: The number of input partial buckets.
///
/// # Constraints
///
/// *   `N_BYTES % 2 == 0`
/// *   `N_HALF_BYTES == N_BYTES / 2`
/// *   `N_ELEMS == N_BYTES / 2 * 8`
/// *   The SIMD lane count of `N_ELEMS` is supported.
trait DualElementsToBytes<const N_BYTES: usize, const N_HALF_BYTES: usize, const N_ELEMS: usize>
where
    LaneCount<N_ELEMS>: SupportedLaneCount,
{
}

/// A type instance represent correspondence between number of partial bucket
/// entries, intermediate data and the output.
///
/// *   `N_BYTES`: The number of bytes in the output.
/// *   `N_HALF_BYTES`: The half of `N_HALF_BYTES`
///     (for intermediate byte-based handling).
/// *   `N_ELEMS`: The number of input partial buckets.
///
/// Those type parameters are constrained by [`DualElementsToBytes`].
struct DualElementsAndBytes<const N_BYTES: usize, const N_HALF_BYTES: usize, const N_ELEMS: usize>;
impl DualElementsToBytes<2, 1, 8> for DualElementsAndBytes<2, 1, 8> {}
impl DualElementsToBytes<4, 2, 16> for DualElementsAndBytes<4, 2, 16> {}
impl DualElementsToBytes<8, 4, 32> for DualElementsAndBytes<8, 4, 32> {}
impl DualElementsToBytes<16, 8, 64> for DualElementsAndBytes<16, 8, 64> {}

/// Data table to interleave two 8-bit integers into an array of dibits.
///
/// This table has two effective indices: `[H][L]`, each denoting high/low bits
/// of the 8 dibits output (2 bytes; in big endian).
///
/// The value can be interpreted as follows:
///
/// ```text
/// H == 0b{H7}{H6}{H5}{H4}{H3}{H2}{H1}{H0}
/// L == 0b{L7}{L6}{L5}{L4}{L3}{L2}{L1}{L0}
/// INTERLEAVE_AS_DIBITS_TABLE[H][L] == [
///     0b{H7}{L7}{H6}{L6}{H5}{L5}{H4}{L4},
///     0b{H3}{L3}{H2}{L2}{H1}{L1}{H0}{L0}
/// ]
/// ```
const INTERLEAVE_AS_DIBITS_TABLE: [[[u8; 2]; 256]; 256] = {
    let mut array = [[[0; 2]; 256]; 256];
    let mut b1 = 0;
    while b1 < 256 {
        let mut b0 = 0;
        while b0 < 256 {
            let mut data = 0u16;
            let mut i = 0;
            while i < 16 {
                if i % 2 == 0 {
                    data |= (((b0 as u16) >> (i / 2)) & 1) << i;
                } else {
                    data |= (((b1 as u16) >> (i / 2)) & 1) << i;
                }
                i += 1;
            }
            array[b1][b0] = data.to_be_bytes(); // always BE
            b0 += 1;
        }
        b1 += 1;
    }
    array
};

/// Aggregate `N_ELEMS` buckets into the `N_BYTES`-byte sub-digest
/// based on three quartiles.
#[inline(always)]
fn sub_aggregation<const N_BYTES: usize, const N_HALF_BYTES: usize, const N_ELEMS: usize>(
    buckets: &[u32; N_ELEMS],
    q1: u32,
    q2: u32,
    q3: u32,
) -> [u8; N_BYTES]
where
    DualElementsAndBytes<N_BYTES, N_HALF_BYTES, N_ELEMS>:
        DualElementsToBytes<N_BYTES, N_HALF_BYTES, N_ELEMS>,
    LaneCount<N_ELEMS>: SupportedLaneCount,
{
    let qv1 = Simd::<u32, N_ELEMS>::splat(q1);
    let qv2 = Simd::<u32, N_ELEMS>::splat(q2);
    let qv3 = Simd::<u32, N_ELEMS>::splat(q3);
    let data = Simd::<u32, N_ELEMS>::from_array(*buckets);
    let qc2 = data.simd_gt(qv2);
    let qb1 = qc2;
    let qb1 = &qb1.to_bitmask().to_le_bytes()[..N_HALF_BYTES]; // always LE
    let qc1 = data.simd_gt(qv1);
    let qc3 = data.simd_gt(qv3);
    let qb0 = qc2 ^ qc1;
    let qb0 = qb0 ^ qc3;
    let qb0 = &qb0.to_bitmask().to_le_bytes()[..N_HALF_BYTES]; // always LE
    let mut out = [0u8; N_BYTES];
    for (out, (&b0, &b1)) in out
        .chunks_exact_mut(2)
        .rev()
        .zip(qb0.iter().zip(qb1.iter()))
    {
        out.copy_from_slice(&INTERLEAVE_AS_DIBITS_TABLE[b1 as usize][b0 as usize]);
    }
    out
}

/// Aggregate 48 buckets into the 12-byte digest based on three quartiles.
///
/// This function requires that:
/// *   `q1 <= q2`
/// *   `q2 <= q3`
#[inline]
pub(super) fn aggregate_48(out: &mut [u8; 12], buckets: &[u32; 48], q1: u32, q2: u32, q3: u32) {
    for (out, subbuckets) in out
        .chunks_mut(4)
        .rev()
        .zip(buckets.as_slice().chunks_exact(4 * 4))
    {
        let subbuckets: [u32; 4 * 4] = subbuckets.try_into().unwrap();
        out.copy_from_slice(&sub_aggregation::<4, { 4 / 2 }, { 4 * 4 }>(
            &subbuckets,
            q1,
            q2,
            q3,
        ));
    }
}

/// Aggregate 128 buckets into the 32-byte digest based on three quartiles.
///
/// This function requires that:
/// *   `q1 <= q2`
/// *   `q2 <= q3`
#[inline]
pub(super) fn aggregate_128(out: &mut [u8; 32], buckets: &[u32; 128], q1: u32, q2: u32, q3: u32) {
    for (out, subbuckets) in out
        .chunks_mut(16)
        .rev()
        .zip(buckets.as_slice().chunks_exact(16 * 4))
    {
        let subbuckets: [u32; 16 * 4] = subbuckets.try_into().unwrap();
        out.copy_from_slice(&sub_aggregation::<16, { 16 / 2 }, { 16 * 4 }>(
            &subbuckets,
            q1,
            q2,
            q3,
        ));
    }
}

/// Aggregate 256 buckets into the 64-byte digest based on three quartiles.
///
/// This function requires that:
/// *   `q1 <= q2`
/// *   `q2 <= q3`
#[inline]
pub(super) fn aggregate_256(out: &mut [u8; 64], buckets: &[u32; 256], q1: u32, q2: u32, q3: u32) {
    for (out, subbuckets) in out
        .chunks_mut(16)
        .rev()
        .zip(buckets.as_slice().chunks_exact(16 * 4))
    {
        let subbuckets: [u32; 16 * 4] = subbuckets.try_into().unwrap();
        out.copy_from_slice(&sub_aggregation::<16, { 16 / 2 }, { 16 * 4 }>(
            &subbuckets,
            q1,
            q2,
            q3,
        ));
    }
}

mod tests;
