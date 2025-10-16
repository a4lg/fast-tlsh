// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! TLSH length comparison.
//!
//! This depends on the difference of encoded 8-bit length specifier.
//! If their [distance (on the ring of modulo 256)](crate::internals::compare::utils::distance_on_ring_mod())
//! is equal to or less than `1`, that value is the distance.  If not, the raw
//! distance `d` is multiplied by the implementation-defined constant:
//! [`12`](length_mult_value!()).
//!
//! Unlike the official implementation, this constant is not designed to be
//! easily configurable in this crate but we usually use this parameter unless
//! you are experimenting with your custom workloads.

use crate::internals::compare::utils::distance_on_ring_mod;

/// The length distance multiplier as an ambiguously-typed literal.
macro_rules! length_mult {
    () => {
        12
    };
}
#[cfg(doc)]
use length_mult as length_mult_value;

/// The maximum distance between two length encodings.
pub const MAX_DISTANCE: u32 = 0x80 * length_mult!();

/// The intermediate type used by [`LDIST_VALUE`].
#[cfg(any(doc, feature = "opt-dist-length-table"))]
type LengthDistanceTableType = u16;

/// Precomputed table for length value distances.
///
/// Since it depends on the wrapped `lvalue1 - lvalue2` (8-bit), we can just
/// create a 256-element table.
///
/// The type of elements in this table is [`LengthDistanceTableType`].
#[cfg(any(doc, feature = "opt-dist-length-table"))]
const LDIST_VALUE: [LengthDistanceTableType; 256] = {
    let mut array = [0; 256];
    let mut i = 0;
    while i < 256 {
        let dist = distance_on_ring_mod(0, i as u8, 0) as LengthDistanceTableType;
        array[i] = if dist <= 1 {
            dist
        } else {
            dist * length_mult!()
        };
        i += 1;
    }
    array
};

/// Computes the distance between two encoded length values.
///
/// Each `lvalue` encodes approximated size of the input and this function
/// takes the difference between two such values.
#[inline(always)]
pub const fn distance(lvalue1: u8, lvalue2: u8) -> u32 {
    cfg_if::cfg_if! {
        if #[cfg(feature = "opt-dist-length-table")] {
            LDIST_VALUE[lvalue1.wrapping_sub(lvalue2) as usize] as u32
        }
        else {
            naive::distance(lvalue1, lvalue2)
        }
    }
}

/// The naïve implementation.
#[cfg(any(test, doc, not(feature = "opt-dist-length-table")))]
pub(crate) mod naive {
    use super::*;

    /// Computes distance between two encoded length values.
    ///
    /// Each `lvalue` encodes approximated size of the input.
    #[inline]
    pub const fn distance(lvalue1: u8, lvalue2: u8) -> u32 {
        let dist = distance_on_ring_mod(lvalue1, lvalue2, 0) as u32;
        if dist <= 1 {
            dist
        } else {
            dist * length_mult!()
        }
    }
}

mod tests;
