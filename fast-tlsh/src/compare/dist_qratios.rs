// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! TLSH Q ratio pair comparison.
//!
//! This module accepts an [`u8`] as a pair of Q ratio values,
//! encoded as lower and upper nibbles (4-bits each).
//!
//! For each of the Q ratio, we calculate [the distance (on the ring of modulo 16)](crate::compare::utils::distance_on_ring_mod()).
//! If this is equal to or less than `1`, that value is the sub-distance.
//! If not, the raw sub-distance `d` is subtracted by 1 and then multiplied by
//! the implementation-defined constant: [`12`](qratio_mult_value!()).
//!
//! The final distance is the sum of two sub-distances.
//!
//! Unlike the official implementation, this constant is not designed to be
//! easily configurable in this crate but we usually use this parameter unless
//! you are experimenting with your custom workloads.

/// The Q ratio distance multiplier as an ambiguously-typed literal.
macro_rules! qratio_mult {
    () => {
        12
    };
}
#[cfg(doc)]
use qratio_mult as qratio_mult_value;

/// The maximum distance between two Q ratio pairs.
pub const MAX_DISTANCE: u32 = 2 * ((0x08 - 1) * qratio_mult!());

/// The intermediate type used by [`QDIST_VALUE`].
#[cfg(any(
    doc,
    all(
        feature = "opt-dist-qratios-table",
        not(feature = "opt-dist-qratios-table-double")
    )
))]
type QRatiosDistanceTableType = u8;

/// Precomputed table for Q ratio value distances.
///
/// This table corresponds to [`naive::sub_distance()`] (16x16 possible values).
///
/// The type of elements in this table is [`QRatiosDistanceTableType`].
#[cfg(any(
    doc,
    all(
        feature = "opt-dist-qratios-table",
        not(feature = "opt-dist-qratios-table-double")
    )
))]
const QDIST_VALUE: [[QRatiosDistanceTableType; 16]; 16] = {
    let mut array = [[0; 16]; 16];
    let mut yi = 0;
    while yi < 16 {
        let y = yi as u8;
        let mut xi = 0;
        while xi < 16 {
            let x = xi as u8;
            array[yi][xi] = naive::sub_distance(x, y) as QRatiosDistanceTableType;
            xi += 1;
        }
        yi += 1;
    }
    array
};

/// The intermediate type used by [`QDIST_VALUE_2`].
#[cfg(any(doc, feature = "opt-dist-qratios-table-double"))]
type QRatiosDistanceTableType2 = u8;

/// Precomputed table for Q ratio pair value distances.
///
/// This table corresponds to [`naive::distance()`] (256x256 possible values).
///
/// The type of elements in this table is [`QRatiosDistanceTableType2`].
#[cfg(any(doc, feature = "opt-dist-qratios-table-double"))]
static QDIST_VALUE_2: [[QRatiosDistanceTableType2; 256]; 256] = {
    let mut array = [[0; 256]; 256];
    let mut yi = 0;
    while yi < 256 {
        let y = yi as u8;
        let mut xi = 0;
        while xi < 256 {
            let x = xi as u8;
            array[yi][xi] = naive::distance(x, y) as QRatiosDistanceTableType2;
            xi += 1;
        }
        yi += 1;
    }
    array
};

/// Computes the distance between two Q ratio pair values.
///
/// Each `qratios` is composed of two Q ratio values (4-bits each) and the sum
/// of the distances of Q ratio values with the same position.
#[inline]
pub fn distance(qratios1: u8, qratios2: u8) -> u32 {
    cfg_if::cfg_if! {
        if #[cfg(feature = "opt-dist-qratios-table-double")] {
            QDIST_VALUE_2[qratios1 as usize][qratios2 as usize] as u32
        }
        else if #[cfg(feature = "opt-dist-qratios-table")] {
            let q1ratio_1 = (qratios1 & 0x0f) as usize;
            let q2ratio_1 = (qratios1 >> 4) as usize;
            let q1ratio_2 = (qratios2 & 0x0f) as usize;
            let q2ratio_2 = (qratios2 >> 4) as usize;
            QDIST_VALUE[q1ratio_1][q1ratio_2] as u32 + QDIST_VALUE[q2ratio_1][q2ratio_2] as u32
        }
        else {
            naive::distance(qratios1, qratios2)
        }
    }
}

/// The naïve implementation.
mod naive {
    use crate::compare::utils::distance_on_ring_mod;

    /// Computes the distance between two Q ratio values.
    ///
    /// This function compares two Q ratio values (`0..16`).
    /// The result of [`distance()`] is the sum of two calls of this function.
    #[inline(always)]
    pub const fn sub_distance(qratio_1: u8, qratio_2: u8) -> u32 {
        let dist = distance_on_ring_mod(qratio_1, qratio_2, 16) as u32;
        if dist <= 1 {
            dist
        } else {
            (dist - 1) * qratio_mult!()
        }
    }

    /// Computes the distance between two Q ratio pair values.
    ///
    /// Each `qratios` is composed of two Q ratio values (4-bits each) and the sum
    /// of [the distances of Q ratio values with the same position](sub_distance()).
    #[cfg(any(
        test,
        doc,
        feature = "opt-dist-qratios-table-double",
        not(feature = "opt-dist-qratios-table")
    ))]
    #[cfg_attr(feature = "unstable", doc(cfg(all())))]
    #[inline]
    pub const fn distance(qratios1: u8, qratios2: u8) -> u32 {
        // Representation as in the internal representation.
        let q1ratio_1 = qratios1 & 0x0f;
        let q2ratio_1 = qratios1 >> 4;
        let q1ratio_2 = qratios2 & 0x0f;
        let q2ratio_2 = qratios2 >> 4;
        sub_distance(q1ratio_1, q1ratio_2) + sub_distance(q2ratio_1, q2ratio_2)
    }
}

mod tests;
