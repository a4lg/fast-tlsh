// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Private bit-manipulation operations

/// Swaps each nibble in the byte.
///
/// This function swaps upper 4 bits (the upper nibble) and lower 4 bits
/// (the lower nibble).
#[cfg(any(
    test,
    doc,
    all(
        feature = "opt-low-memory-hex-str-encode-half-table",
        not(feature = "opt-low-memory-hex-str-encode-min-table")
    )
))]
pub fn swap_nibble_in_u8(value: u8) -> u8 {
    value.rotate_left(4)
}

/// Naïve bit manipulation implementations.
#[cfg(any(doc, test))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
mod naive {
    /// Swaps each nibble in the byte.
    ///
    /// This function swaps upper 4 bits (the upper nibble) and lower 4 bits
    /// (the lower nibble).
    pub fn swap_nibble_in_u8(value: u8) -> u8 {
        ((value >> 4) & 0x0f) | ((value & 0x0f) << 4)
    }
}

mod tests;
