// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Hexadecimal string utilities.

#[cfg(all(
    feature = "opt-low-memory-hex-str-encode-half-table",
    not(feature = "opt-low-memory-hex-str-encode-min-table")
))]
use crate::parse::bits::swap_nibble_in_u8;

/// The uppercase hexadecimal digit array.
const HEX_UPPER_NIBBLE_TABLE: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];

/// The array to encode a byte to two uppercase hexadecimal digits.
#[cfg(any(doc, not(feature = "opt-low-memory-hex-str-encode-min-table")))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_UPPER_BYTE_TABLE: [[u8; 2]; 256] = {
    let mut array = [[0; 2]; 256];
    let mut i = 0;
    while i < 256 {
        array[i] = [
            HEX_UPPER_NIBBLE_TABLE[i >> 4],
            HEX_UPPER_NIBBLE_TABLE[i & 0x0f],
        ];
        i += 1;
    }
    array
};

/// The array to encode a byte to two uppercase hexadecimal digits (in reverse nibble).
#[cfg(any(doc, not(feature = "opt-low-memory-hex-str-encode-half-table")))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_UPPER_BYTE_REV_TABLE: [[u8; 2]; 256] = {
    let mut array = HEX_UPPER_BYTE_TABLE;
    let mut i = 0;
    while i < 256 {
        // Swap high digit and low digit.
        (array[i][0], array[i][1]) = (array[i][1], array[i][0]);
        i += 1;
    }
    array
};

/// The type of [`HEX_INVALID`] and [`HEX_REV_TABLE_LO`].
#[cfg(not(feature = "opt-low-memory-hex-str-decode-quarter-table"))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
type HexDecodeTableType = u16;

/// Invalid hexadecimal character (mask or value, depending on the configuration).
///
/// The type of this value is [`HexDecodeTableType`].
#[cfg(not(feature = "opt-low-memory-hex-str-decode-quarter-table"))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_INVALID: HexDecodeTableType = 0x100;

/// Hexadecimal character-to-value (and validness) table (for low nibble).
///
/// The type of elements in this table is [`HexDecodeTableType`].
#[rustfmt::skip]
#[cfg(not(feature = "opt-low-memory-hex-str-decode-quarter-table"))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_REV_TABLE_LO: [HexDecodeTableType; 256] = [
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x000, 0x001, 0x002, 0x003, 0x004, 0x005, 0x006, 0x007, 0x008, 0x009, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x00a, 0x00b, 0x00c, 0x00d, 0x00e, 0x00f, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x00a, 0x00b, 0x00c, 0x00d, 0x00e, 0x00f, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
    0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100, 0x100,
];

/// Hexadecimal character-to-value (and validness) table (for hi nibble).
///
/// The type of elements in this table is [`HexDecodeTableType`].
#[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_REV_TABLE_HI: [HexDecodeTableType; 256] = {
    let mut array = [0; 256];
    let mut i = 0;
    while i < 256 {
        let x = HEX_REV_TABLE_LO[i];
        array[i] = if x == HEX_INVALID { x } else { x << 4 };
        i += 1;
    }
    array
};

/// The type of [`HEX_INVALID`] and [`HEX_REV_TABLE_LO`].
#[cfg(feature = "opt-low-memory-hex-str-decode-quarter-table")]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
type HexDecodeTableType = u8;

/// Invalid hexadecimal character (mask or value, depending on the configuration).
///
/// The type of this value is [`HexDecodeTableType`].
#[cfg(feature = "opt-low-memory-hex-str-decode-quarter-table")]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_INVALID: HexDecodeTableType = 0xff;

/// Hexadecimal character-to-value (and validness) table (for low nibble).
///
/// The type of elements in this table is [`HexDecodeTableType`].
#[cfg(all(
    feature = "opt-low-memory-hex-str-decode-quarter-table",
    not(feature = "opt-low-memory-hex-str-decode-min-table")
))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
const HEX_REV_TABLE_LO: [HexDecodeTableType; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

/// Converts a hexadecimal digit to an [`u8`] value.
///
/// If the conversion fails, it returns the fixed value `0xff`.
///
/// Note that this kind of implementation is notoriously bad
/// for branch prediction.
#[cfg(any(doc, test, feature = "opt-low-memory-hex-str-decode-min-table"))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
#[inline]
pub(super) const fn decode_digit(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'A'..=b'F' => digit - b'A' + 10,
        b'a'..=b'f' => digit - b'a' + 10,
        // HEX_INVALID must be equal to 0xff when used outside tests.
        _ => 0xff,
    }
}
#[cfg(feature = "opt-low-memory-hex-str-decode-quarter-table")]
static_assertions::const_assert_eq!(HEX_INVALID, 0xff);

/// Converts length 2 hexadecimal string (with normal nibble endianness)
/// to an [`u8`] value.
///
/// If the conversion fails, it returns [`None`].
#[cfg(any(test, doc, not(feature = "opt-simd-parse-hex")))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
#[inline(always)]
pub fn decode_1(src: &[u8]) -> Option<u8> {
    if src.len() != 2 {
        return None;
    }
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))] {
            let value = HEX_REV_TABLE_HI[src[0] as usize] | HEX_REV_TABLE_LO[src[1] as usize];
            if value & HEX_INVALID != 0 {
                None
            } else {
                Some(value as u8)
            }
        } else if #[cfg(not(feature = "opt-low-memory-hex-str-decode-quarter-table"))] {
            let value_hi = HEX_REV_TABLE_LO[src[0] as usize];
            let value_lo = HEX_REV_TABLE_LO[src[1] as usize];
            let value = value_hi << 4 | value_lo;
            if value >= HEX_INVALID {
                None
            } else {
                Some(value as u8)
            }
        } else if #[cfg(not(feature = "opt-low-memory-hex-str-decode-min-table"))] {
            let value_hi = HEX_REV_TABLE_LO[src[0] as usize];
            let value_lo = HEX_REV_TABLE_LO[src[1] as usize];
            if (value_lo == HEX_INVALID) || (value_hi == HEX_INVALID) {
                None
            } else {
                Some(value_hi << 4 | value_lo)
            }
        } else {
            let value_hi = decode_digit(src[0]);
            let value_lo = decode_digit(src[1]);
            if (value_lo == HEX_INVALID) || (value_hi == HEX_INVALID) {
                None
            } else {
                Some(value_hi << 4 | value_lo)
            }
        }
    }
}

/// Converts a hexadecimal string (with normal nibble endianness)
/// to an array of [`u8`].
///
/// It returns whether this function has succeeded.
/// If not, `dst` may be partially written (or may be not).
#[cfg(any(test, doc, not(feature = "opt-simd-parse-hex")))]
#[cfg_attr(feature = "unstable", doc(cfg(all())))]
#[inline]
pub fn decode_array<const N: usize>(dst: &mut [u8; N], src: &[u8]) -> bool {
    if src.len() != N * 2 {
        return false;
    }
    for (dst, src) in dst.iter_mut().zip(src.chunks_exact(2)) {
        let value = decode_1(src);
        if let Some(value) = value {
            *dst = value;
        } else {
            return false;
        }
    }
    true
}

/// Converts length 2 hexadecimal string (with "reverse" nibble endianness)
/// to an [`u8`] value.
///
/// If the conversion fails, it returns [`None`].
#[inline(always)]
pub fn decode_rev_1(src: &[u8]) -> Option<u8> {
    if src.len() != 2 {
        return None;
    }
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))] {
            let value = HEX_REV_TABLE_LO[src[0] as usize] | HEX_REV_TABLE_HI[src[1] as usize];
            if value & HEX_INVALID != 0 {
                None
            } else {
                Some(value as u8)
            }
        } else if #[cfg(not(feature = "opt-low-memory-hex-str-decode-quarter-table"))] {
            let value_lo = HEX_REV_TABLE_LO[src[0] as usize];
            let value_hi = HEX_REV_TABLE_LO[src[1] as usize];
            let value = value_hi << 4 | value_lo;
            if value >= HEX_INVALID {
                None
            } else {
                Some(value as u8)
            }
        } else if #[cfg(not(feature = "opt-low-memory-hex-str-decode-min-table"))] {
            let value_lo = HEX_REV_TABLE_LO[src[0] as usize];
            let value_hi = HEX_REV_TABLE_LO[src[1] as usize];
            if (value_lo == HEX_INVALID) || (value_hi == HEX_INVALID) {
                None
            } else {
                Some(value_hi << 4 | value_lo)
            }
        } else {
            let value_lo = decode_digit(src[0]);
            let value_hi = decode_digit(src[1]);
            if (value_lo == HEX_INVALID) || (value_hi == HEX_INVALID) {
                None
            } else {
                Some(value_hi << 4 | value_lo)
            }
        }
    }
}

/// Converts a hexadecimal string (with "reverse" nibble endianness)
/// to an array of [`u8`].
///
/// It returns whether this function has succeeded.
/// If not, `dst` may be partially written (or may be not).
#[inline]
pub fn decode_rev_array<const N: usize>(dst: &mut [u8; N], src: &[u8]) -> bool {
    if src.len() != N * 2 {
        return false;
    }
    for (dst, src) in dst.iter_mut().zip(src.chunks_exact(2)) {
        let value = decode_rev_1(src);
        if let Some(value) = value {
            *dst = value;
        } else {
            return false;
        }
    }
    true
}

/// Convert an [`u8`] array into a hexadecimal string (without reverse nibble conversion).
#[cfg(not(feature = "opt-simd-convert-hex"))]
#[inline]
pub fn encode_array<const N: usize>(dst: &mut [u8], src: &[u8; N]) {
    for (dst, &value) in dst.chunks_exact_mut(2).zip(src.iter()) {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "opt-low-memory-hex-str-encode-min-table"))] {
                dst.copy_from_slice(&HEX_UPPER_BYTE_TABLE[value as usize]);
            } else {
                dst[0] = HEX_UPPER_NIBBLE_TABLE[(value >> 4) as usize];
                dst[1] = HEX_UPPER_NIBBLE_TABLE[(value & 0x0f) as usize];
            }
        }
    }
}

/// Convert an [`u8`] value into a hexadecimal string (with reverse nibble conversion).
#[inline(always)]
pub fn encode_rev_1(dst: &mut [u8], value: u8) {
    assert!(dst.len() >= 2);
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "opt-low-memory-hex-str-encode-half-table"))] {
            dst[0..2].copy_from_slice(&HEX_UPPER_BYTE_REV_TABLE[value as usize]);
        } else if #[cfg(not(feature = "opt-low-memory-hex-str-encode-min-table"))] {
            dst[0..2].copy_from_slice(&HEX_UPPER_BYTE_TABLE[swap_nibble_in_u8(value) as usize]);
        } else {
            dst[0] = HEX_UPPER_NIBBLE_TABLE[(value & 0x0f) as usize];
            dst[1] = HEX_UPPER_NIBBLE_TABLE[(value >> 4) as usize];
        }
    }
}

/// Convert an [`u8`] array into a hexadecimal string (with reverse nibble conversion).
#[inline]
pub fn encode_rev_array<const N: usize>(dst: &mut [u8], src: &[u8; N]) {
    for (dst, &value) in dst.chunks_exact_mut(2).zip(src.iter()) {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "opt-low-memory-hex-str-encode-half-table"))] {
                dst.copy_from_slice(&HEX_UPPER_BYTE_REV_TABLE[value as usize]);
            } else if #[cfg(not(feature = "opt-low-memory-hex-str-encode-min-table"))] {
                dst.copy_from_slice(&HEX_UPPER_BYTE_TABLE[swap_nibble_in_u8(value) as usize]);
            } else {
                dst[0] = HEX_UPPER_NIBBLE_TABLE[(value & 0x0f) as usize];
                dst[1] = HEX_UPPER_NIBBLE_TABLE[(value >> 4) as usize];
            }
        }
    }
}

mod tests;
