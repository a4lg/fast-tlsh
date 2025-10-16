// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::pearson`].

#![cfg(test)]

use super::{
    INITIAL_STATE, SUBST_TABLE, final_48, final_256, init, tlsh_b_mapping_48, tlsh_b_mapping_256,
    update, update_double,
};

#[test]
fn init_example() {
    assert_eq!(init(0x02), 0x31);
}

#[test]
fn init_and_update_equivalence() {
    for value in u8::MIN..=u8::MAX {
        assert_eq!(init(value), update(INITIAL_STATE, value));
    }
}

#[test]
fn update_sample() {
    let state = init(0x02);
    let state = update(state, 0xbe);
    let state = update(state, 0xef);
    assert_eq!(state, 0x63);
}

#[test]
fn update_double_equivalence() {
    for salt in u8::MIN..=u8::MAX {
        for b1 in u8::MIN..=u8::MAX {
            for b2 in u8::MIN..=u8::MAX {
                assert_eq!(update_double(salt, b1, b2), update(update(salt, b1), b2));
            }
        }
    }
}

#[test]
fn final_256_example() {
    let state = init(0x02);
    let state = update_double(state, 0xbe, 0xef);
    let state = final_256(state, 0x00);
    assert_eq!(state, 0x4b);
}

#[test]
fn final_48_example() {
    let state = init(0x02);
    let state = update_double(state, 0xbe, 0xef);
    let state = final_48(state, 0x00);
    assert_eq!(state, 0x1b);
}

#[test]
fn final_48_and_256() {
    for state in u8::MIN..=u8::MAX {
        for value in u8::MIN..=u8::MAX {
            let expected = {
                let v = final_256(state, value);
                if v >= 240 { 48 } else { v % 48 }
            };
            assert_eq!(final_48(state, value), expected);
        }
    }
}

#[test]
fn tlsh_b_mapping_examples() {
    // See also: final_256_example
    assert_eq!(tlsh_b_mapping_256(0x02, 0xbe, 0xef, 0x00), 0x4b);
    // See also: final_48_example
    assert_eq!(tlsh_b_mapping_48(0x02, 0xbe, 0xef, 0x00), 0x1b);
}

#[test]
fn subst_table_on_tlsh_optimization() {
    // All of these examples are from TLSH's tlsh_impl.cpp (as a part of
    // "manual constant folding on the first byte" we are not performing).
    macro_rules! test_case {
        ($initial: expr, $expected: expr) => {
            assert_eq!(init($initial), $expected);
            assert_eq!(SUBST_TABLE[$initial], $expected);
        };
    }
    test_case!(0, 1);
    test_case!(2, 49);
    test_case!(3, 12);
    test_case!(5, 178);
    test_case!(7, 166);
    test_case!(11, 84);
    test_case!(13, 230);
    test_case!(17, 197);
    test_case!(19, 181);
    test_case!(23, 80);
    test_case!(29, 142);
    test_case!(31, 200);
    test_case!(37, 253);
    test_case!(41, 101);
    test_case!(43, 18);
    test_case!(47, 222);
    test_case!(53, 237);
    test_case!(59, 214);
    test_case!(61, 227);
    test_case!(67, 22);
    test_case!(71, 175);
    test_case!(73, 5);
}
