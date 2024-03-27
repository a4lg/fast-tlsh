// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! TLSH body comparison.
//!
//! The distance of two TLSH bodies is the sum of quartile distances.
//! For each quartile values (2-bits each), we take an absolute difference.
//! If the raw quartile distance is the maximum (i.e. if one is `0b00` and
//! the another is `0b11`), the raw quartile distance (`3`) is replaced with the
//! implementation-defined constant: [`6`](BODY_OUTLIER_VALUE).
//!
//! Unlike the official implementation, this constant is not designed to be
//! easily configurable in this crate but we usually use this parameter unless
//! you are experimenting with your custom workloads.
//!
//! Not only that, SIMD and pseudo-SIMD implementations assume that this value
//! equals to `6` (logical / arithmetic expression suitable for SIMD is found
//! by machine assuming this constant and will drastically change depending
//! on this constant).
//!
//! For the internal algorithm, see
//! [SIMD-friendly TLSH Body Distance Calculation](crate::_docs::internal_simd_dist_body).

#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    feature = "detect-features",
    feature = "unstable",
    target_arch = "arm",
    target_feature = "v7"
))]
use std::arch::is_arm_feature_detected;
#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    feature = "detect-features",
    any(target_arch = "x86", target_arch = "x86_64")
))]
use std::arch::is_x86_feature_detected;
#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    feature = "detect-features",
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        all(target_arch = "arm", feature = "unstable", target_feature = "v7")
    )
))]
use std::sync::OnceLock;

mod arm_neon;
#[allow(dead_code)]
mod portable_simd;
#[allow(dead_code)]
mod pseudo_simd_32;
#[allow(dead_code)]
mod pseudo_simd_64;
mod x86_avx2;
mod x86_sse2;
mod x86_sse4_1;

mod fuzzer;

/// The body outlier value when the difference is the maximum (`0b11`).
pub const BODY_OUTLIER_VALUE: u32 = 6;
static_assertions::const_assert!(BODY_OUTLIER_VALUE >= 0b11); // must be at least 3.

/// The maximum distance between two 12-byte bodies.
pub const MAX_DISTANCE_SHORT: u32 = 12 * 4 * BODY_OUTLIER_VALUE;

/// The maximum distance between two 32-byte bodies.
pub const MAX_DISTANCE_NORMAL: u32 = 32 * 4 * BODY_OUTLIER_VALUE;

/// The maximum distance between two 64-byte bodies.
pub const MAX_DISTANCE_LONG: u32 = 64 * 4 * BODY_OUTLIER_VALUE;

/// 32-byte variant of the distance computation function.
///
/// By default, this is a reference to either [`pseudo_simd_64::distance_32()`]
/// or [`pseudo_simd_32::distance_32()`].
///
/// If the platform is detected to have specific features (e.g. SIMD
/// instructions), this is overridden with a reference to the suitable function
/// (or its wrapper).
#[allow(clippy::type_complexity)]
#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    feature = "detect-features",
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        all(target_arch = "arm", feature = "unstable", target_feature = "v7")
    )
))]
#[cfg_attr(
    feature = "unstable",
    doc(cfg(all(
        feature = "simd-per-arch",
        feature = "opt-simd-body-comparison",
        feature = "detect-features"
    )))
)]
static DISPATCH_DISTANCE_32: OnceLock<&'static (dyn Fn(&[u8; 32], &[u8; 32]) -> u32 + Sync)> =
    OnceLock::new();

/// 64-byte variant of the distance computation function.
///
/// By default, this is a reference to either [`pseudo_simd_64::distance_64()`]
/// or [`pseudo_simd_32::distance_64()`].
///
/// If the platform is detected to have specific features (e.g. SIMD
/// instructions), this is overridden with a reference to the suitable function
/// (or its wrapper).
#[allow(clippy::type_complexity)]
#[cfg(all(
    feature = "simd-per-arch",
    feature = "opt-simd-body-comparison",
    feature = "detect-features",
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        all(target_arch = "arm", feature = "unstable", target_feature = "v7")
    )
))]
#[cfg_attr(
    feature = "unstable",
    doc(cfg(all(
        feature = "simd-per-arch",
        feature = "opt-simd-body-comparison",
        feature = "detect-features"
    )))
)]
static DISPATCH_DISTANCE_64: OnceLock<&'static (dyn Fn(&[u8; 64], &[u8; 64]) -> u32 + Sync)> =
    OnceLock::new();

/// Generates distance functions like [`distance_32()`].
///
/// Note that is doesn't generate [`distance_12()`] (the shortest variant)
/// because handling this variant using SIMD can be very inefficient.
macro_rules! distance_func_template {
    {$($name:ident = ($size:literal, $dispatch:path);)*} => {
        $(
            #[doc = concat!("Computes the distance between two ", stringify!($size), "-byte TLSH bodies.")]
            #[inline]
            pub fn $name(body1: &[u8; $size], body2: &[u8; $size]) -> u32 {
                cfg_if::cfg_if! {
                    if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        feature = "detect-features",
                        any(
                            target_arch = "x86",
                            target_arch = "x86_64",
                            all(target_arch = "arm", feature = "unstable", target_feature = "v7")
                        )
                    ))] {
                        // Detect runtime CPU features, cache and call
                        $dispatch.get_or_init(|| {
                            #[cfg(all(target_arch = "arm"))]
                            {
                                if is_arm_feature_detected!("neon") {
                                    return &|body1, body2| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            arm_neon::$name(body1, body2)
                                        }
                                    };
                                }
                            }
                            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                            {
                                if is_x86_feature_detected!("avx2") {
                                    return &|body1, body2| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_avx2::$name(body1, body2)
                                        }
                                    };
                                }
                                if is_x86_feature_detected!("sse4.1") {
                                    return &|body1, body2| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_sse4_1::$name(body1, body2)
                                        }
                                    };
                                }
                                if is_x86_feature_detected!("sse2") {
                                    return &|body1, body2| {
                                        #[allow(unsafe_code)]
                                        unsafe {
                                            x86_sse2::$name(body1, body2)
                                        }
                                    };
                                }
                            }
                            if usize::BITS >= 64 {
                                &pseudo_simd_64::$name
                            } else {
                                &pseudo_simd_32::$name
                            }
                        })(body1, body2)
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        target_arch = "aarch64",
                        target_feature = "neon"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            arm_neon::$name(body1, body2)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        target_arch = "arm",
                        feature = "unstable",
                        target_feature = "v7",
                        target_feature = "neon"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            arm_neon::$name(body1, body2)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "avx2"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_avx2::$name(body1, body2)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "sse4.1"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_sse4_1::$name(body1, body2)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-per-arch",
                        feature = "opt-simd-body-comparison",
                        any(target_arch = "x86", target_arch = "x86_64"),
                        target_feature = "sse2"
                    ))] {
                        #[allow(unsafe_code)]
                        unsafe {
                            x86_sse2::$name(body1, body2)
                        }
                    }
                    else if #[cfg(all(
                        feature = "simd-portable",
                        feature = "opt-simd-body-comparison"
                    ))] {
                        portable_simd::$name(body1, body2)
                    }
                    else {
                        if usize::BITS >= 64 {
                            pseudo_simd_64::$name(body1, body2)
                        } else {
                            pseudo_simd_32::$name(body1, body2)
                        }
                    }
                }
            }
        )*
    }
}

distance_func_template! {
    distance_32 = (32, DISPATCH_DISTANCE_32);
    distance_64 = (64, DISPATCH_DISTANCE_64);
}

/// Computes the distance between two 12-byte TLSH bodies.
#[cfg_attr(feature = "unstable", coverage(off))]
pub fn distance_12(body1: &[u8; 12], body2: &[u8; 12]) -> u32 {
    if usize::BITS >= 64 {
        pseudo_simd_64::distance_12(body1, body2)
    } else {
        pseudo_simd_32::distance_12(body1, body2)
    }
}

/// The naïve implementation.
#[cfg(any(doc, test))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
pub(crate) mod naive {
    /// Computes the distance between two dibits.
    pub fn distance_dibits(x: u8, y: u8) -> u32 {
        assert!(x < 4);
        assert!(y < 4);
        let diff = u32::abs_diff(x as u32, y as u32);
        if diff == 0b11 {
            super::BODY_OUTLIER_VALUE
        } else {
            diff
        }
    }

    /// Computes the distance between two TLSH bodies (in variable length).
    pub fn distance<const N: usize>(body1: &[u8; N], body2: &[u8; N]) -> u32 {
        body1
            .iter()
            .zip(body2.iter())
            .map(|(&x, &y)| {
                (0..4u32)
                    .map(move |i| {
                        // Extract each dibit (0b00-0b11) and take abs(x-y)
                        let nx = (x >> (i * 2)) & 0b11;
                        let ny = (y >> (i * 2)) & 0b11;
                        distance_dibits(nx, ny)
                    })
                    .sum::<u32>()
            })
            .sum::<u32>()
    }
}

mod tests;
