// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Bucket aggregation based on quartiles.
//!
//! The [`aggregate_48()`], [`aggregate_128()`] and [`aggregate_256()`]
//! functions convert specified number of buckets (an array of [`u32`]) to
//! the array of [`u8`] (with the 1/4 size of the buckets) based on quartile
//! values.
//!
//! Normally, the 128 bucket variant [`aggregate_128()`] is used.
//!
//! # Algorithm
//!
//! Each bucket value is converted into a dibit by following criteria:
//!
//! Value | Meaning
//! ----- | ---------------------------------------------------------------
//!  `11` | Exceeds (greater than) 25-percentile value from the top (`q3`)
//!  `10` | Exceeds (greater than) 50-percentile value from the top (`q2`)
//!  `01` | Exceeds (greater than) 75-percentile value from the top (`q1`)
//!  `00` | Does not satisfy any of those.
//!
//! Then, they are arranged so that the hexadecimal representation of the byte
//! array corresponds to a *big-endian* integer corresponding bits 0–1 to the
//! bucket 0, bits 2–3 to the bucket 1 and so on (i.e. the *last* byte
//! represents the *first* 4 buckets).
//!
//! Note that, all functions require that:
//!
//! *   `q1 <= q2`
//! *   `q2 <= q3`
//!
//! # Inevitable Unbalance
//!
//! Despite that constraints above and that `q1` through `q3` represent quartile
//! values in reality, we still cannot guarantee that we can have the same
//! number of `0b00` through `0b11` dibit entries because some buckets have
//! the same amount (which is close to a quartile value).
//!
//! To show a quite extreme example, following 50 byte sequence:
//!
//! ```text
//! 000000 59 c7 b0 e5 47 be 4c 06 dc 95 03 c5 16 47 2f 8d  >Y...G.L......G/.<
//! 000010 03 ea 73 d1 c0 b8 79 cd 09 87 b9 1f df f9 7c db  >..s...y.......|.<
//! 000020 38 76 d7 f2 04 de c2 cf 9f 7f ab f0 d5 7a 11 56  >8v...........z.V<
//! 000030 f1 89                                            >..<
//! ```
//!
//! generates weird fuzzy hash like this with an 128 buckets configuration:
//!
//! ```text
//! T11C90440000000000000000000000000000000000000000000000000000000000000000
//! ```
//!
//! This is caused because we have the same amount in all 128 buckets (`1`)
//! after processing the file above and we cannot exceed any of quartile values
//! (all `1`s, making all dibits `0b00`).
//!
//! # Testing
//!
//! `q1`, `q2` and `q3` on the tests are not necessarily constrained to the
//! exact quartile values (computed from the buckets itself) but subject to
//! the constraint: `q1 <= q2 <= q3`.
//!
//! So, all algorithms do not depend on following facts
//! (that are all satisfied on TLSH):
//!
//! *   `q1`, `q2` and `q3` represents exact quartile values.
//! *   There is a bucket that have the same amount as `q1`, `q2` or `q3`.

#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-bucket-aggregation",
    feature = "detect-features",
    any(target_arch = "x86", target_arch = "x86_64")
))]
use std::arch::is_x86_feature_detected;
#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-bucket-aggregation",
    feature = "detect-features",
    any(target_arch = "x86", target_arch = "x86_64")
))]
use std::sync::OnceLock;

#[allow(dead_code)]
mod portable_simd;
mod wasm32_simd128;
mod x86_avx2;
mod x86_sse2;
mod x86_ssse3;

#[cfg(all(test, feature = "tests-slow"))]
mod fuzzer;

/// The naïve implementation.
#[allow(dead_code)]
pub(crate) mod naive {
    /// Get a quartile value.
    ///
    /// This function converts `value` to a dibit as follows:
    ///
    ///  Value | Meaning
    /// ------ | -------------------------------
    /// `0b11` | Exceeds `q3` (`q3 > value`)
    /// `0b10` | Exceeds `q2` (`q2 > value`)
    /// `0b01` | Exceeds `q1` (`q1 > value`)
    /// `0b00` | Does not satisfy any of those.
    ///
    /// This function requires that:
    ///
    /// *   `q1 <= q2`
    /// *   `q2 <= q3`
    #[inline(always)]
    pub(super) const fn get_quartile(value: u32, q1: u32, q2: u32, q3: u32) -> u8 {
        debug_assert!(q1 <= q2);
        debug_assert!(q2 <= q3);
        if value > q3 {
            3
        } else if value > q2 {
            2
        } else if value > q1 {
            1
        } else {
            0
        }
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
                #[inline]
                pub fn $name(out: &mut [u8; $size_small], buckets: &[u32; $size_large], q1: u32, q2: u32, q3: u32) {
                    for (out, subbuckets) in out.iter_mut().rev().zip(buckets.as_slice().chunks_exact(4)) {
                        *out = subbuckets.iter().rev().fold(0u8, |x, &b| {
                            let q = get_quartile(b, q1, q2, q3);
                            x << 2 | q
                        });
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
}

/// Generates aggregation functions like [`aggregate_128()`].
macro_rules! aggregation_func_template {
    {$($name:ident = ($size_small:literal, $size_large:literal, $dispatch:ident);)*} => {
        $(
            #[doc = concat!(
                stringify!($size_large),
                "-bucket aggregation function (to be dynamically dispatched).\n",
                "\n",
                "By default, this is a reference to [`naive::aggregate_",
                stringify!($size_large),
                "()`].\n",
                "\n",
                "If the platform is detected to have specific features ",
                "(e.g. SIMD instructions), this is overridden with a reference to the ",
                "suitable function (or its wrapper)."
            )]
            #[allow(clippy::type_complexity)]
            #[cfg(all(
                feature = "simd-per-arch",
                feature = "opt-simd-bucket-aggregation",
                feature = "detect-features",
                any(target_arch = "x86", target_arch = "x86_64")
            ))]
            #[cfg_attr(
                feature = "unstable",
                doc(cfg(all(
                    feature = "simd-per-arch",
                    feature = "opt-simd-bucket-aggregation",
                    feature = "detect-features"
                )))
            )]
            static $dispatch: OnceLock<
                &'static (dyn Fn(&mut [u8; $size_small], &[u32; $size_large], u32, u32, u32) + Sync),
            > = OnceLock::new();

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
            #[inline]
            pub fn $name(out: &mut [u8; $size_small], buckets: &[u32; $size_large], q1: u32, q2: u32, q3: u32) {
                debug_assert!(q1 <= q2);
                debug_assert!(q2 <= q3);
                cfg_if::cfg_if! {
                    if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-bucket-aggregation",
                        feature = "detect-features",
                        any(target_arch = "x86", target_arch = "x86_64")
                    ))] {
                        // Detect runtime CPU features, cache and call
                        $dispatch.get_or_init(|| {
                            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                            {
                                if is_x86_feature_detected!("avx2") {
                                    return &|out, buckets, q1, q2, q3| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_avx2::$name(out, buckets, q1, q2, q3)
                                        }
                                    };
                                }
                                if is_x86_feature_detected!("ssse3") {
                                    return &|out, buckets, q1, q2, q3| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_ssse3::$name(out, buckets, q1, q2, q3)
                                        }
                                    };
                                }
                                if is_x86_feature_detected!("sse2") {
                                    return &|out, buckets, q1, q2, q3| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_sse2::$name(out, buckets, q1, q2, q3)
                                        }
                                    };
                                }
                            }
                            &naive::$name
                        })(out, buckets, q1, q2, q3)
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-bucket-aggregation",
                        not(feature = "detect-features"),
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "avx2"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_avx2::$name(out, buckets, q1, q2, q3)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-bucket-aggregation",
                        not(feature = "detect-features"),
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "ssse3"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_ssse3::$name(out, buckets, q1, q2, q3)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-bucket-aggregation",
                        not(feature = "detect-features"),
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "sse2"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_sse2::$name(out, buckets, q1, q2, q3)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-bucket-aggregation",
                        target_arch = "wasm32",
                        target_feature = "simd128"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            wasm32_simd128::$name(out, buckets, q1, q2, q3)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-portable",
                        feature = "opt-simd-bucket-aggregation"
                    ))] {
                        portable_simd::$name(out, buckets, q1, q2, q3)
                    }
                    else {
                        naive::$name(out, buckets, q1, q2, q3)
                    }
                }
            }
        )*
    }
}

aggregation_func_template! {
    aggregate_48  = (12,  48, DISPATCH_AGGREGATE_48);
    aggregate_128 = (32, 128, DISPATCH_AGGREGATE_128);
    aggregate_256 = (64, 256, DISPATCH_AGGREGATE_256);
}

mod tests;
