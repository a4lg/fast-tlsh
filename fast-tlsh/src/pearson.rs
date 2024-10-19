// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Pearson hashing and the TLSH's B (bucket) mapping.
//!
//! # Warning
//!
//! This module is going to be private on the next major version.
//!
//! # Summary
//!
//! See [Pearson, 1990 (doi:10.1145/78973.78978)](https://doi.org/10.1145%2F78973.78978)
//! and the [Wikipedia article](https://en.wikipedia.org/wiki/Pearson_hashing)
//! for details.
//!
//! # On TLSH
//!
//! TLSH's official implementation often use 4-byte Peason hash updates.
//! For instance, we use following 4 bytes to select the bucket to update:
//!
//! 1.  A prime (constant)
//! 2.  The latest byte
//! 3.  An old byte in the sliding window
//! 4.  (likewise)
//!
//! And on short fuzzy hashes, it uses the special transformation (that requires
//! a special substitution table [for finalization](final_48())).
//!
//! We don't implement "manual constant folding on the first byte" optimization
//! as seen in the `fast_b_mapping` function in the original implementation
//! (also see `b_mapping` to see the difference) because LLVM is smart enough
//! to perform the equivalent.
//!
//! Consider following snippets are equivalent except in short fuzzy hashes:
//!
//! ```text
//! // TLSH (unoptimized virtual example)
//! b_mapping(2,  a4, a3, a2)
//! // TLSH (manually optimized; excerpt from tlsh_impl.cpp)
//! // 49 is the result after processing the first byte (a prime): 2.
//! fast_b_mapping(49,  a4, a3, a2)
//! // fast-tlsh (this crate; internal)
//! final_256(update_double(init(0x02), a4, a3), a2)
//! // fast-tlsh (this crate; public)
//! pearson::tlsh_b_mapping_256(0x02, a4, a3, a2)
//! ```
//!
//! On short fuzzy hashes:
//!
//! ```text
//! // fast-tlsh (this crate; internal)
//! final_48(update_double(init(0x02), a4, a3), a2)
//! // fast-tlsh (this crate; public)
//! pearson::tlsh_b_mapping_48(0x02, a4, a3, a2)
//! ```

/// The initial state of Pearson hashing.
pub const INITIAL_STATE: u8 = 0;

/// The substitution table for Pearson hashing.
pub const SUBST_TABLE: [u8; 256] = [
    0x01, 0x57, 0x31, 0x0c, 0xb0, 0xb2, 0x66, 0xa6, 0x79, 0xc1, 0x06, 0x54, 0xf9, 0xe6, 0x2c, 0xa3,
    0x0e, 0xc5, 0xd5, 0xb5, 0xa1, 0x55, 0xda, 0x50, 0x40, 0xef, 0x18, 0xe2, 0xec, 0x8e, 0x26, 0xc8,
    0x6e, 0xb1, 0x68, 0x67, 0x8d, 0xfd, 0xff, 0x32, 0x4d, 0x65, 0x51, 0x12, 0x2d, 0x60, 0x1f, 0xde,
    0x19, 0x6b, 0xbe, 0x46, 0x56, 0xed, 0xf0, 0x22, 0x48, 0xf2, 0x14, 0xd6, 0xf4, 0xe3, 0x95, 0xeb,
    0x61, 0xea, 0x39, 0x16, 0x3c, 0xfa, 0x52, 0xaf, 0xd0, 0x05, 0x7f, 0xc7, 0x6f, 0x3e, 0x87, 0xf8,
    0xae, 0xa9, 0xd3, 0x3a, 0x42, 0x9a, 0x6a, 0xc3, 0xf5, 0xab, 0x11, 0xbb, 0xb6, 0xb3, 0x00, 0xf3,
    0x84, 0x38, 0x94, 0x4b, 0x80, 0x85, 0x9e, 0x64, 0x82, 0x7e, 0x5b, 0x0d, 0x99, 0xf6, 0xd8, 0xdb,
    0x77, 0x44, 0xdf, 0x4e, 0x53, 0x58, 0xc9, 0x63, 0x7a, 0x0b, 0x5c, 0x20, 0x88, 0x72, 0x34, 0x0a,
    0x8a, 0x1e, 0x30, 0xb7, 0x9c, 0x23, 0x3d, 0x1a, 0x8f, 0x4a, 0xfb, 0x5e, 0x81, 0xa2, 0x3f, 0x98,
    0xaa, 0x07, 0x73, 0xa7, 0xf1, 0xce, 0x03, 0x96, 0x37, 0x3b, 0x97, 0xdc, 0x5a, 0x35, 0x17, 0x83,
    0x7d, 0xad, 0x0f, 0xee, 0x4f, 0x5f, 0x59, 0x10, 0x69, 0x89, 0xe1, 0xe0, 0xd9, 0xa0, 0x25, 0x7b,
    0x76, 0x49, 0x02, 0x9d, 0x2e, 0x74, 0x09, 0x91, 0x86, 0xe4, 0xcf, 0xd4, 0xca, 0xd7, 0x45, 0xe5,
    0x1b, 0xbc, 0x43, 0x7c, 0xa8, 0xfc, 0x2a, 0x04, 0x1d, 0x6c, 0x15, 0xf7, 0x13, 0xcd, 0x27, 0xcb,
    0xe9, 0x28, 0xba, 0x93, 0xc6, 0xc0, 0x9b, 0x21, 0xa4, 0xbf, 0x62, 0xcc, 0xa5, 0xb4, 0x75, 0x4c,
    0x8c, 0x24, 0xd2, 0xac, 0x29, 0x36, 0x9f, 0x08, 0xb9, 0xe8, 0x71, 0xc4, 0xe7, 0x2f, 0x92, 0x78,
    0x33, 0x41, 0x1c, 0x90, 0xfe, 0xdd, 0x5d, 0xbd, 0xc2, 0x8b, 0x70, 0x2b, 0x47, 0x6d, 0xb8, 0xd1,
];

/// The substitution table for 2 bytes of Pearson hashing.
///
/// Note that the first index denotes the byte 2 (not 1) to maximize
/// address calculation efficiency.
#[cfg(any(doc, feature = "opt-pearson-table-double"))]
#[allow(clippy::large_const_arrays)]
const SUBST_TABLE_DOUBLE: [[u8; 256]; 256] = {
    // TODO:
    // Since making this table static now is a breaking change,
    // do following on the next major version:
    //  1.  Remove #[allow] above
    //  2.  Remove `const` from `update_double()` function below and
    //  3.  Make this item `static` rather than `const`.
    let mut array = [[0; 256]; 256];
    let mut b2 = 0;
    while b2 < 256 {
        let mut b1 = 0;
        while b1 < 256 {
            array[b2][b1] = SUBST_TABLE[SUBST_TABLE[b1] as usize ^ b2];
            b1 += 1;
        }
        b2 += 1;
    }
    array
};

/// The special substitution table for 48-bucket variant of TLSH.
///
/// For each [`SUBST_TABLE`] value (`x`), this is:
///
/// *   `x % 48` (when `x < 240`)
/// *   `48`     (otherwise)
///
/// It avoids bias on the bucket distribution (because 256 values makes an
/// uneven distribution (`256 % 48 != 0`), they only use first
/// `256 / 48 * 48 == 240` values for bucket counting).
///
/// Instead, it increases the bias on the checksum (because values other than
/// `48` will get intermediate frequency of `5/256` but `48` gets `16/256`;
/// `256 / 48 == 5`, `256 - 256 / 48 * 48 == 16`).
const SUBST_TABLE_48: [u8; 256] = {
    let mut array = SUBST_TABLE;
    let mut i = 0;
    while i < 256 {
        if array[i] >= 240 {
            array[i] = 48;
        } else {
            array[i] %= 48;
        }
        i += 1;
    }
    array
};

/// Process one byte (as a initialization) using Pearson hashing.
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// let state = pearson::init(0x02);
/// assert_eq!(state, 0x31);
/// # }
/// ```
#[inline(always)]
pub const fn init(value: u8) -> u8 {
    update(INITIAL_STATE, value)
}

/// Process one byte using Pearson hashing.
///
/// # Examples
///
/// ## Usage
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// let state = pearson::init(0x02);
/// let state = pearson::update(state, 0xbe);
/// let state = pearson::update(state, 0xef);
/// assert_eq!(state, 0x63);
/// # }
/// ```
///
/// ## Relation with [`init()`]
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// // init() is equivalent to updating 1 byte from the initial state.
/// let state1 = pearson::init(0x02);
/// let state2 = pearson::INITIAL_STATE;
/// let state2 = pearson::update(state2, 0x02);
/// assert_eq!(state1, state2);
/// # }
/// ```
#[inline(always)]
pub const fn update(state: u8, value: u8) -> u8 {
    SUBST_TABLE[(state ^ value) as usize]
}

/// Process two bytes using Pearson hashing.
///
/// This function updates the Pearson hashing state with two bytes:
/// `b1` and `b2`.
///
/// This is equivalent to two calls to [`update()`] but may be optimized
/// for faster processing.
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// let state = pearson::init(0x02);
/// let state1 = pearson::update(pearson::update(state, 0xbe), 0xef);
/// let state2 = pearson::update_double(state, 0xbe, 0xef);
/// assert_eq!(state1, state2);
/// # }
/// ```
#[inline(always)]
pub const fn update_double(state: u8, b1: u8, b2: u8) -> u8 {
    cfg_if::cfg_if! {
        if #[cfg(feature = "opt-pearson-table-double")] {
            SUBST_TABLE_DOUBLE[b2 as usize][(state ^ b1) as usize]
        }
        else {
            update(update(state, b1), b2)
        }
    }
}

/// Process one byte using Pearson hashing for 256-bucket finalization.
///
/// On the 256-bucket variant, this is the same as regular [`update()`].
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// let state = pearson::init(0x02);
/// let state = pearson::update_double(state, 0xbe, 0xef);
/// let state = pearson::final_256(state, 0x00);
/// assert_eq!(state, 0x4b);
/// # }
/// ```
#[inline(always)]
pub const fn final_256(state: u8, value: u8) -> u8 {
    update(state, value)
}

/// Process one byte using Pearson hashing for 48-bucket finalization.
///
/// Assuming that the return value of [`final_256()`] is `x`,
/// the return value of this function is as follows:
///
/// *   `x % 48` (when `x < 240`)
/// *   `48`     (otherwise)
///
/// It avoids bias on the bucket distribution (because 256 values makes an
/// uneven distribution (`256 % 48 != 0`), they only use first
/// `256 / 48 * 48 == 240` values for bucket counting).
///
/// Instead, it increases the bias on the checksum (because values other than
/// `48` will get intermediate frequency of `5/256` but `48` gets `16/256`;
/// `256 / 48 == 5`, `256 - 256 / 48 * 48 == 16`).
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// let state = pearson::init(0x02);
/// let state = pearson::update_double(state, 0xbe, 0xef);
/// let state = pearson::final_48(state, 0x00);
/// assert_eq!(state, 0x1b);
/// # }
/// ```
#[inline(always)]
pub const fn final_48(state: u8, value: u8) -> u8 {
    SUBST_TABLE_48[(state ^ value) as usize]
}

/// TLSH's B (bucket) mapping on the 256-bucket variant.
///
/// On TLSH, the first byte `b0` is a constant (a prime when updating the
/// internal bucket and `0` when updating the internal checksum).
///
/// On the 256-bucket variant, this is the same as updating 4 bytes: `b0`
/// through `b3` (in that order) from the initial state.
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// assert_eq!(pearson::tlsh_b_mapping_256(0x02, 0xbe, 0xef, 0x00), 0x4b);
/// # }
/// ```
#[inline(always)]
pub fn tlsh_b_mapping_256(b0: u8, b1: u8, b2: u8, b3: u8) -> u8 {
    final_256(update_double(init(b0), b1, b2), b3)
}

/// TLSH's B (bucket) mapping on the 48-bucket variant.
///
/// On TLSH, the first byte `b0` is a constant (a prime when updating the
/// internal bucket and `0` when updating the internal checksum).
///
/// Assuming that the return value of [`tlsh_b_mapping_256()`] is `x`,
/// the return value of this function is as follows:
///
/// *   `x % 48` (when `x < 240`)
/// *   `48`     (otherwise)
///
/// It avoids bias on the bucket distribution (because 256 values makes an
/// uneven distribution (`256 % 48 != 0`), they only use first
/// `256 / 48 * 48 == 240` values for bucket counting).
///
/// Instead, it increases the bias on the checksum (because values other than
/// `48` will get intermediate frequency of `5/256` but `48` gets `16/256`;
/// `256 / 48 == 5`, `256 - 256 / 48 * 48 == 16`).
///
/// # Example
///
/// ```
/// // Requires the `experiment-pearson` feature.
/// # #[cfg(feature = "experiment-pearson")] {
/// use tlsh::pearson;
///
/// assert_eq!(pearson::tlsh_b_mapping_48(0x02, 0xbe, 0xef, 0x00), 0x1b);
/// # }
/// ```
#[inline(always)]
pub fn tlsh_b_mapping_48(b0: u8, b1: u8, b2: u8, b3: u8) -> u8 {
    final_48(update_double(init(b0), b1, b2), b3)
}

mod tests;
