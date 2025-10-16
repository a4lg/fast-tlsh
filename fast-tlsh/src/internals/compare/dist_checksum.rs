// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! TLSH checksum comparison.
//!
//! Calculation of this distance is quite simple.
//! For each checksum byte, `0` if they are the same and `1` if not.
//!
//! We usually use 1-byte checksum and the range of the distance is `0..=1`.
//! If we use 3-byte checksum, the range is `0..=3`.  If we don't have any
//! checksum bytes, the distance is always `0`.

/// Computes the distance between two 1-byte checksum values.
#[inline(always)]
pub const fn distance_1(checksum1: [u8; 1], checksum2: [u8; 1]) -> u32 {
    if checksum1[0] != checksum2[0] { 1 } else { 0 }
}

/// Computes the distance between two 3-byte checksum values.
#[inline(always)]
pub const fn distance_3(checksum1: [u8; 3], checksum2: [u8; 3]) -> u32 {
    let mut sum = 0;
    let mut i = 0;
    while i < 3 {
        sum += if checksum1[i] != checksum2[i] { 1 } else { 0 };
        i += 1;
    }
    sum
}

#[cfg(test)]
pub(crate) mod generic {
    /// Computes distance on the ring of integer modulo `n`.
    ///
    /// This function calculates distance between `x` and `y` on the ring
    /// of integer modulo `n` (except `n` is zero; in this case, this is handled
    /// as `T::max + 1`).
    ///
    /// `T` must be a primitive unsigned integer type.
    ///
    /// This is the `mod_diff` function in the original paper and many compatible
    /// implementations.
    #[inline]
    pub fn distance<const N: usize>(checksum1: [u8; N], checksum2: [u8; N]) -> u32 {
        let mut sum = 0;
        for (&x, &y) in checksum1.iter().zip(checksum2.iter()) {
            sum += if x != y { 1 } else { 0 };
        }
        sum
    }
}

mod tests;
