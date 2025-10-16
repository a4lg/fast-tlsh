// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::compare::dist_qratios`].

#![cfg(test)]

use super::naive;

#[cfg(all(
    feature = "opt-dist-qratios-table",
    not(feature = "opt-dist-qratios-table-double")
))]
use super::QRatiosDistanceTableType;
#[cfg(feature = "opt-dist-qratios-table-double")]
use super::QRatiosDistanceTableType2;

#[test]
fn arithmetic_correctness_naive() {
    // No arithmetic overflow occurs on the naïve implementation.
    // 0x08 is the maximum value of mod_diff(x, y, 16).
    assert!(
        0x08u32
            .checked_sub(1)
            .and_then(|x| x.checked_mul(2))
            .and_then(|x| x.checked_mul(qratio_mult!()))
            .is_some()
    );
}

#[cfg(all(
    feature = "opt-dist-qratios-table",
    not(feature = "opt-dist-qratios-table-double")
))]
#[allow(clippy::useless_conversion)]
#[test]
fn table_consistency()
where
    QRatiosDistanceTableType: From<u8>,
    u32: From<QRatiosDistanceTableType>,
{
    // Above constraints make sures that u8 ⊆ QRatiosDistanceTableType ⊆ u32.
    // 0x08 is the maximum value of mod_diff(x, y, 16).
    let dist = QRatiosDistanceTableType::from(0x08u8);
    assert!(
        dist.checked_sub(1)
            .and_then(|x| x.checked_mul(qratio_mult!()))
            .is_some()
    );
}

#[cfg(feature = "opt-dist-qratios-table-double")]
#[allow(clippy::useless_conversion)]
#[test]
fn table_consistency_double()
where
    QRatiosDistanceTableType2: From<u8>,
    u32: From<QRatiosDistanceTableType2>,
{
    // Above constraints make sures that u8 ⊆ QRatiosDistanceTableType2 ⊆ u32.
    // 0x08 is the maximum value of mod_diff(x, y, 16).
    let dist = QRatiosDistanceTableType2::from(0x08u8);
    assert!(
        dist.checked_sub(1)
            .and_then(|x| x.checked_mul(2))
            .and_then(|x| x.checked_mul(qratio_mult!()))
            .is_some()
    );
}

#[test]
fn equivalence_optimized_impl() {
    for qratios2 in u8::MIN..=u8::MAX {
        for qratios1 in u8::MIN..=u8::MAX {
            assert_eq!(
                super::distance(qratios1, qratios2),
                naive::distance(qratios1, qratios2)
            );
        }
    }
}
