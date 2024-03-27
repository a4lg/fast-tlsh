// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Serde Tests.

#![cfg(test)]

use std::str::FromStr;

use tlsh::{FuzzyHashType, Tlsh};

#[test]
fn serde_json_example() {
    const HASH_STR: &str =
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488";
    let hash_str_quoted: String = format!(r#""{HASH_STR}""#);
    let hash = Tlsh::from_str(HASH_STR).unwrap();
    assert_eq!(serde_json::to_string(&hash).unwrap(), hash_str_quoted);
    let hash2 = serde_json::from_str::<Tlsh>(hash_str_quoted.as_str()).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn serde_json_de_err_not_a_hash() {
    let result = serde_json::from_str::<Tlsh>("1");
    assert!(result.is_err());
}

#[test]
fn serde_json_de_err_invalid_str() {
    // Invalid prefix (T0)
    let result = serde_json::from_str::<Tlsh>(
        r#""T02AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488""#,
    );
    assert!(result.is_err());
}

#[test]
fn postcard_example() {
    const HASH_STR: &str =
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488";
    // 0x23 (data length) is prefixed
    const POSTCARD_DATA: &[u8] = b"\x23\
        \xa2\x5d\xeb\
        \x86\xff\xe4\x1d\x17\xcc\x26\x88\x76\xa9\xae\x47\x20\x77\xb2\xb0\
        \x03\x27\x16\xdb\xaf\x18\x49\xa7\x64\x7d\xdb\x7c\x0d\xf1\x64\x88";
    let hash = Tlsh::from_str(HASH_STR).unwrap();
    let mut bytes = [0u8; Tlsh::SIZE_IN_BYTES];
    assert_eq!(hash.store_into_bytes(&mut bytes), Ok(Tlsh::SIZE_IN_BYTES));
    assert_eq!(bytes.as_slice(), &POSTCARD_DATA[1..]);
    let data = postcard::to_stdvec(&hash).unwrap();
    assert_eq!(data.as_slice(), POSTCARD_DATA);
    let hash2 = postcard::from_bytes::<Tlsh>(data.as_slice()).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn postcard_de_err_not_a_hash() {
    const POSTCARD_DATA: &[u8] = b"\x00"; // empty byte string
    let result = postcard::from_bytes::<Tlsh>(POSTCARD_DATA);
    assert!(result.is_err());
}

#[test]
fn ciborium_example() {
    let mut data = vec![];
    const HASH_STR: &str =
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488";
    // 0x58 0x23 (byte array with 1-byte length: 0x23) is prefixed
    const CBOR_DATA: &[u8] = b"\x58\x23\
        \xa2\x5d\xeb\
        \x86\xff\xe4\x1d\x17\xcc\x26\x88\x76\xa9\xae\x47\x20\x77\xb2\xb0\
        \x03\x27\x16\xdb\xaf\x18\x49\xa7\x64\x7d\xdb\x7c\x0d\xf1\x64\x88";
    let hash = Tlsh::from_str(HASH_STR).unwrap();
    ciborium::into_writer(&hash, &mut data).expect("failed to write");
    assert_eq!(data.as_slice(), CBOR_DATA);
    let hash2 = ciborium::from_reader::<Tlsh, _>(data.as_slice()).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn ciborium_de_err_not_a_byte_array() {
    const CBOR_DATA: &[u8] = b"\x00"; // non-negative integer zero
    let result = ciborium::from_reader::<Tlsh, _>(CBOR_DATA);
    assert!(result.is_err());
}

#[test]
fn ciborium_de_err_empty_byte_array() {
    const CBOR_DATA: &[u8] = b"\x40"; // empty bytes
    let result = ciborium::from_reader::<Tlsh, _>(CBOR_DATA);
    assert!(result.is_err());
}

#[cfg(feature = "serde-buffered")]
#[test]
fn ciborium_example_buffered() {
    const HASH_STR: &str =
        "T12AD5BE86FFE41D17CC268876A9AE472077B2B0032716DBAF1849A7647DDB7C0DF16488";
    let hash = Tlsh::from_str(HASH_STR).unwrap();
    // An Indefinite-Length Byte String
    // (Ciborium requires buffering to deserialize from such strings)
    const CBOR_DATA: &[u8] = b"\x5f\
        \x43\xa2\x5d\xeb\
        \x50\x86\xff\xe4\x1d\x17\xcc\x26\x88\x76\xa9\xae\x47\x20\x77\xb2\xb0\
        \x50\x03\x27\x16\xdb\xaf\x18\x49\xa7\x64\x7d\xdb\x7c\x0d\xf1\x64\x88\
        \xff";
    let hash2 = ciborium::from_reader::<Tlsh, _>(CBOR_DATA).unwrap();
    assert_eq!(hash, hash2);
}
