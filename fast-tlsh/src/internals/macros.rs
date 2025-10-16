// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2023-2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! The internal macros.

#![allow(unused_macros)]

/// "Optionally" unsafe block.
///
/// When this crate is built with the `unsafe` feature, this macro is
/// expanded to an `unsafe` block.
///
/// Inside this block, you may place statements that may change the behavior
/// depending on the feature `unsafe`.  For instance, you may place
/// [`invariant!()`] inside this block.
///
/// ```ignore
/// # // Because this is an internal macro, we must ignore on the doctest
/// # // because each Rust doctest's scope is external to this crate.
/// // INTERNAL USE (INSIDE THIS CRATE) ONLY
/// // let index: usize = ... (but proven to be inside the array).
/// # let index = 3usize;
/// let array = [0, 1, 2, 3];
/// optionally_unsafe! {
///     invariant!(index < array.len());
/// }
/// // Bound checking may be optimized out.
/// let result = array[index];
/// ```
#[doc(alias = "optionally_unsafe")]
macro_rules! optionally_unsafe_impl {
    {$($tokens: tt)*} => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "unsafe")] {
                unsafe { $($tokens)* }
            } else {
                { $($tokens)* }
            }
        }
    };
}

/// Declare an invariant for optimization.
///
/// When the feature `unsafe` is disabled, it only places [`debug_assert!()`].
/// If `unsafe` is enabled, [`core::hint::assert_unchecked()`] is used.
///
/// Optimization behaviors are disabled on tests.
///
/// Use this macro along with [`optionally_unsafe!{}`].
#[doc(alias = "invariant")]
macro_rules! invariant_impl {
    ($expr: expr) => {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "unsafe", not(test)))] {
                core::hint::assert_unchecked($expr);
            } else {
                debug_assert!($expr);
            }
        }
    };
}

pub(crate) use invariant_impl as invariant;
pub(crate) use optionally_unsafe_impl as optionally_unsafe;

mod tests;
