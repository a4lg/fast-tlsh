// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! Utility for measuring distance between two sub-data items.

/// Computes distance on the ring of integer modulo `n`.
///
/// This function calculates the distance between `x` and `y` on the ring
/// of integer modulo `n` (except `n` is zero; in this case, this is handled
/// as `256`, which is [`u8::MAX`]` + 1`).
///
/// This is the `mod_diff` function in the original paper and many compatible
/// implementations (although all of them I've seen use `256` instead of `0`).
#[inline]
pub const fn distance_on_ring_mod(x: u8, y: u8, n: u8) -> u8 {
    debug_assert!(n == 0 || x < n);
    debug_assert!(n == 0 || y < n);
    // Swapping (dl, dr) on the either side helps optimization.
    let (dl, dr) = if x >= y {
        (x.wrapping_sub(y), y.wrapping_add(n).wrapping_sub(x))
    } else {
        (x.wrapping_add(n).wrapping_sub(y), y.wrapping_sub(x))
    };
    // Take the minimum (because u8::min is unavailable in the constant context)
    if dl <= dr {
        dl
    } else {
        dr
    }
}

/// The generic implementation.
#[cfg(test)]
pub(crate) mod generic {
    use core::num::Wrapping;
    use num_traits::Unsigned;

    /// Computes distance on the ring of integer modulo `n`.
    ///
    /// This function calculates the distance between `x` and `y` on the ring
    /// of integer modulo `n` (except `n` is zero; in this case, this is handled
    /// as `T::MAX + 1`).
    ///
    /// `T` must be a primitive unsigned integer type.
    ///
    /// This is the `mod_diff` function in the original paper and many compatible
    /// implementations.
    #[inline]
    pub fn distance_on_ring_mod<T>(x: T, y: T, n: T) -> T
    where
        T: PartialEq + Ord + Unsigned,
        Wrapping<T>: Copy + Ord + Unsigned,
    {
        debug_assert!(n == T::zero() || x < n);
        debug_assert!(n == T::zero() || y < n);
        let x = Wrapping(x);
        let y = Wrapping(y);
        let n = Wrapping(n);
        // Swapping (dl, dr) on the either side helps optimization.
        let (dl, dr) = if x >= y {
            (x - y, y + n - x)
        } else {
            (x + n - y, y - x)
        };
        core::cmp::min(dl.0, dr.0)
    }
}

mod tests;
