// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::parse::hex_str`].

#![cfg(test)]

use super::{
    HEX_UPPER_NIBBLE_TABLE, decode_1, decode_array, decode_digit, decode_rev_1, decode_rev_array,
    encode_rev_1, encode_rev_array,
};

#[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
use super::HEX_REV_TABLE_HI;
#[cfg(not(feature = "opt-low-memory-hex-str-decode-min-table"))]
use super::{HEX_INVALID, HEX_REV_TABLE_LO, HexDecodeTableType};

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::internals::parse::bits::swap_nibble_in_u8;

#[test]
fn hex_table_examples() {
    #[cfg(not(feature = "opt-low-memory-hex-str-decode-min-table"))]
    {
        assert_eq!(HEX_REV_TABLE_LO[b'0' as usize], 0x00);
        assert_eq!(HEX_REV_TABLE_LO[b'1' as usize], 0x01);
        assert_eq!(HEX_REV_TABLE_LO[b'2' as usize], 0x02);
        assert_eq!(HEX_REV_TABLE_LO[b'3' as usize], 0x03);
        assert_eq!(HEX_REV_TABLE_LO[b'4' as usize], 0x04);
        assert_eq!(HEX_REV_TABLE_LO[b'5' as usize], 0x05);
        assert_eq!(HEX_REV_TABLE_LO[b'6' as usize], 0x06);
        assert_eq!(HEX_REV_TABLE_LO[b'7' as usize], 0x07);
        assert_eq!(HEX_REV_TABLE_LO[b'8' as usize], 0x08);
        assert_eq!(HEX_REV_TABLE_LO[b'9' as usize], 0x09);
        assert_eq!(HEX_REV_TABLE_LO[b'a' as usize], 0x0a);
        assert_eq!(HEX_REV_TABLE_LO[b'b' as usize], 0x0b);
        assert_eq!(HEX_REV_TABLE_LO[b'c' as usize], 0x0c);
        assert_eq!(HEX_REV_TABLE_LO[b'd' as usize], 0x0d);
        assert_eq!(HEX_REV_TABLE_LO[b'e' as usize], 0x0e);
        assert_eq!(HEX_REV_TABLE_LO[b'f' as usize], 0x0f);
        assert_eq!(HEX_REV_TABLE_LO[b'A' as usize], 0x0a);
        assert_eq!(HEX_REV_TABLE_LO[b'B' as usize], 0x0b);
        assert_eq!(HEX_REV_TABLE_LO[b'C' as usize], 0x0c);
        assert_eq!(HEX_REV_TABLE_LO[b'D' as usize], 0x0d);
        assert_eq!(HEX_REV_TABLE_LO[b'E' as usize], 0x0e);
        assert_eq!(HEX_REV_TABLE_LO[b'F' as usize], 0x0f);
    }
    #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
    {
        assert_eq!(HEX_REV_TABLE_HI[b'0' as usize], 0x00);
        assert_eq!(HEX_REV_TABLE_HI[b'1' as usize], 0x10);
        assert_eq!(HEX_REV_TABLE_HI[b'2' as usize], 0x20);
        assert_eq!(HEX_REV_TABLE_HI[b'3' as usize], 0x30);
        assert_eq!(HEX_REV_TABLE_HI[b'4' as usize], 0x40);
        assert_eq!(HEX_REV_TABLE_HI[b'5' as usize], 0x50);
        assert_eq!(HEX_REV_TABLE_HI[b'6' as usize], 0x60);
        assert_eq!(HEX_REV_TABLE_HI[b'7' as usize], 0x70);
        assert_eq!(HEX_REV_TABLE_HI[b'8' as usize], 0x80);
        assert_eq!(HEX_REV_TABLE_HI[b'9' as usize], 0x90);
        assert_eq!(HEX_REV_TABLE_HI[b'a' as usize], 0xa0);
        assert_eq!(HEX_REV_TABLE_HI[b'b' as usize], 0xb0);
        assert_eq!(HEX_REV_TABLE_HI[b'c' as usize], 0xc0);
        assert_eq!(HEX_REV_TABLE_HI[b'd' as usize], 0xd0);
        assert_eq!(HEX_REV_TABLE_HI[b'e' as usize], 0xe0);
        assert_eq!(HEX_REV_TABLE_HI[b'f' as usize], 0xf0);
        assert_eq!(HEX_REV_TABLE_HI[b'A' as usize], 0xa0);
        assert_eq!(HEX_REV_TABLE_HI[b'B' as usize], 0xb0);
        assert_eq!(HEX_REV_TABLE_HI[b'C' as usize], 0xc0);
        assert_eq!(HEX_REV_TABLE_HI[b'D' as usize], 0xd0);
        assert_eq!(HEX_REV_TABLE_HI[b'E' as usize], 0xe0);
        assert_eq!(HEX_REV_TABLE_HI[b'F' as usize], 0xf0);
    }
}

#[cfg(not(feature = "opt-low-memory-hex-str-decode-min-table"))]
#[test]
fn hex_table_exhaustive() {
    let mut hash_lo = HashMap::new();
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))] {
            let mut hash_hi = HashMap::new();
        }
    }
    for &ch in &HEX_UPPER_NIBBLE_TABLE {
        assert_eq!(ch, ch.to_ascii_uppercase());
        // Uppercase or a decimal digit
        {
            assert_eq!(
                HexDecodeTableType::from_str_radix(std::str::from_utf8(&[b'0', ch]).unwrap(), 16),
                Ok(HEX_REV_TABLE_LO[ch as usize])
            );
            hash_lo.insert(ch, HEX_REV_TABLE_LO[ch as usize]);
        }
        #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
        {
            assert_eq!(
                HexDecodeTableType::from_str_radix(std::str::from_utf8(&[ch, b'0']).unwrap(), 16),
                Ok(HEX_REV_TABLE_HI[ch as usize])
            );
            hash_hi.insert(ch, HEX_REV_TABLE_HI[ch as usize]);
        }
        // Lowercase or a decimal digit
        let ch = ch.to_ascii_lowercase();
        {
            assert_eq!(
                HexDecodeTableType::from_str_radix(std::str::from_utf8(&[b'0', ch]).unwrap(), 16),
                Ok(HEX_REV_TABLE_LO[ch as usize])
            );
            hash_lo.insert(ch, HEX_REV_TABLE_LO[ch as usize]);
        }
        #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
        {
            assert_eq!(
                HexDecodeTableType::from_str_radix(std::str::from_utf8(&[ch, b'0']).unwrap(), 16),
                Ok(HEX_REV_TABLE_HI[ch as usize])
            );
            hash_hi.insert(ch, HEX_REV_TABLE_HI[ch as usize]);
        }
    }
    for ch in u8::MIN..=u8::MAX {
        if !hash_lo.contains_key(&ch) {
            assert_eq!(HEX_REV_TABLE_LO[ch as usize], HEX_INVALID);
            #[cfg(not(feature = "opt-low-memory-hex-str-decode-half-table"))]
            {
                assert_eq!(HEX_REV_TABLE_HI[ch as usize], HEX_INVALID);
            }
        }
    }
}

#[test]
fn decode_digit_examples() {
    assert_eq!(decode_digit(b'0'), 0x0);
    assert_eq!(decode_digit(b'1'), 0x1);
    assert_eq!(decode_digit(b'2'), 0x2);
    assert_eq!(decode_digit(b'3'), 0x3);
    assert_eq!(decode_digit(b'4'), 0x4);
    assert_eq!(decode_digit(b'5'), 0x5);
    assert_eq!(decode_digit(b'6'), 0x6);
    assert_eq!(decode_digit(b'7'), 0x7);
    assert_eq!(decode_digit(b'8'), 0x8);
    assert_eq!(decode_digit(b'9'), 0x9);
    assert_eq!(decode_digit(b'a'), 0xa);
    assert_eq!(decode_digit(b'b'), 0xb);
    assert_eq!(decode_digit(b'c'), 0xc);
    assert_eq!(decode_digit(b'd'), 0xd);
    assert_eq!(decode_digit(b'e'), 0xe);
    assert_eq!(decode_digit(b'f'), 0xf);
    assert_eq!(decode_digit(b'A'), 0xa);
    assert_eq!(decode_digit(b'B'), 0xb);
    assert_eq!(decode_digit(b'C'), 0xc);
    assert_eq!(decode_digit(b'D'), 0xd);
    assert_eq!(decode_digit(b'E'), 0xe);
    assert_eq!(decode_digit(b'F'), 0xf);
}

#[test]
fn decode_digit_exhaustive() {
    let mut hash = HashMap::new();
    for i in 0x0..=0xf {
        let s = format!("{i:01x}");
        assert_eq!(s.len(), 1);
        // Insert lowercase entry
        let s = s.as_bytes()[0];
        let v = decode_digit(s.to_ascii_lowercase());
        assert_eq!(v, i);
        assert_eq!(hash.insert(s, v), None);
        // Insert uppercase entry
        let s = s.to_ascii_uppercase();
        let v = decode_digit(s);
        match hash.entry(s) {
            Entry::Occupied(x) => {
                assert_eq!(v, *x.get());
            }
            Entry::Vacant(x) => {
                x.insert(v);
            }
        }
    }
    // Other digits are all invalid.
    for ch in u8::MIN..=u8::MAX {
        match hash.entry(ch) {
            Entry::Occupied(_) => {
                assert!(decode_digit(ch) < 0x10);
            }
            Entry::Vacant(_) => {
                assert_eq!(decode_digit(ch), 0xff);
            }
        }
    }
}

#[test]
fn hex_digits_for_encode() {
    for &ch in &HEX_UPPER_NIBBLE_TABLE {
        // To match to the official implementation,
        // hexadecimal digits must be upper case.
        assert!(ch.is_ascii_digit() || ch.is_ascii_uppercase());
    }
}

#[test]
fn decode_1_examples() {
    assert_eq!(decode_1(b"12"), Some(0x12));
    assert_eq!(decode_1(b"0f"), Some(0x0f));
    assert_eq!(decode_1(b"A5"), Some(0xa5));
    assert_eq!(decode_1(b"g0"), None);
    assert_eq!(decode_1(b"0g"), None);
}

#[test]
fn decode_1_fail_len() {
    // The length of the input must be 2.
    assert_eq!(decode_1(b"1"), None);
    assert_eq!(decode_1(b"111"), None);
}

#[test]
fn decode_array_examples() {
    let mut array = [0u8; 8];
    // Accepts both lower case and upper case values.
    assert!(decode_array(&mut array, b"0123456789abcdef"));
    assert_eq!(&array, b"\x01\x23\x45\x67\x89\xab\xcd\xef");
    assert!(decode_array(&mut array, b"0123456789ABCDEF"));
    assert_eq!(&array, b"\x01\x23\x45\x67\x89\xab\xcd\xef");
}

#[test]
fn decode_array_fail_len() {
    let mut array = [0u8; 8];
    // 8 byte buffer requires 16 byte input but 14 is given here.
    assert!(!decode_array(&mut array, b"0123456789abcd"));
}

#[test]
fn decode_array_fail_data() {
    let mut array = [0u8; 8];
    // Invalid digit '@' is given.
    assert!(!decode_array(&mut array, b"0123456@89abcdef"));
}

#[test]
fn decode_rev_1_examples() {
    assert_eq!(decode_rev_1(b"12"), Some(0x21));
    assert_eq!(decode_rev_1(b"0f"), Some(0xf0));
    assert_eq!(decode_rev_1(b"A5"), Some(0x5a));
    assert_eq!(decode_rev_1(b"g0"), None);
    assert_eq!(decode_rev_1(b"0g"), None);
}

#[test]
fn decode_rev_1_exhaustive() {
    for value in u8::MIN..=u8::MAX {
        let swapped = swap_nibble_in_u8(value);
        let upper = format!("{swapped:02X}");
        let lower = format!("{swapped:02x}");
        assert_eq!(decode_rev_1(upper.as_bytes()), Some(value));
        assert_eq!(decode_rev_1(lower.as_bytes()), Some(value));
    }
}

#[test]
fn decode_rev_1_fail_len() {
    // The length of the input must be 2.
    assert_eq!(decode_rev_1(b"1"), None);
    assert_eq!(decode_rev_1(b"111"), None);
}

#[test]
fn decode_rev_array_examples() {
    let mut array = [0u8; 8];
    // Accepts both lower case and upper case values.
    assert!(decode_rev_array(&mut array, b"0123456789abcdef"));
    assert_eq!(&array, b"\x10\x32\x54\x76\x98\xba\xdc\xfe");
    assert!(decode_rev_array(&mut array, b"0123456789ABCDEF"));
    assert_eq!(&array, b"\x10\x32\x54\x76\x98\xba\xdc\xfe");
}

#[test]
fn decode_rev_array_fail_len() {
    let mut array = [0u8; 8];
    // 8 byte buffer requires 16 byte input but 14 is given here.
    assert!(!decode_rev_array(&mut array, b"0123456789abcd"));
}

#[test]
fn decode_rev_array_fail_data() {
    let mut array = [0u8; 8];
    // Invalid digit '@' is given.
    assert!(!decode_rev_array(&mut array, b"0123456@89abcdef"));
}

#[test]
fn encode_rev_1_example() {
    let mut dst = [0u8; 2];
    encode_rev_1(dst.as_mut(), 0x5a);
    assert_eq!(dst.as_slice(), b"A5");
}

#[test]
fn encode_rev_1_example_with_excess_bytes() {
    let mut dst = [0xffu8; 4];
    encode_rev_1(dst.as_mut(), 0x5a);
    // Excess part of the buffer is kept.
    assert_eq!(dst.as_slice(), b"A5\xff\xff");
}

#[test]
fn encode_rev_1_exhaustive() {
    for value in u8::MIN..=u8::MAX {
        let swapped = swap_nibble_in_u8(value);
        let expected = format!("{swapped:02X}");
        let mut dst = [0u8; 2];
        encode_rev_1(dst.as_mut(), value);
        assert_eq!(expected.as_bytes(), dst.as_slice());
    }
}

#[test]
#[should_panic]
fn encode_rev_1_insufficient_buffer() {
    let mut dst = [0u8; 1];
    encode_rev_1(dst.as_mut(), 0x5a);
}

#[test]
fn encode_rev_array_example() {
    let mut dst = [0u8; 8 * 2];
    encode_rev_array(
        dst.as_mut(),
        &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );
    assert_eq!(dst.as_slice(), b"1032547698BADCFE");
}
