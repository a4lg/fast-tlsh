// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The internal intrinsics.

use cfg_if::cfg_if;

/// Hints to the compiler that branch condition is likely to be true.
///
/// This is a thin wrapper to [`core::hint::likely()`] and requires
/// `#![feature(likely_unlikely)]` when the `unstable` feature is enabled.
#[inline(always)]
pub(crate) fn likely(value_likely_to_be_true: bool) -> bool {
    cfg_if! {
        if #[cfg(feature = "unstable")] {
            core::hint::likely(value_likely_to_be_true)
        }
        else {
            value_likely_to_be_true
        }
    }
}

/// Hints to the compiler that branch condition is unlikely to be true.
///
/// This is a thin wrapper to [`core::hint::unlikely()`] and requires
/// `#![feature(likely_unlikely)]` when the `unstable` feature is enabled.
#[inline(always)]
pub(crate) fn unlikely(value_unlikely_to_be_true: bool) -> bool {
    cfg_if! {
        if #[cfg(feature = "unstable")] {
            core::hint::unlikely(value_unlikely_to_be_true)
        }
        else {
            value_unlikely_to_be_true
        }
    }
}

mod tests;
