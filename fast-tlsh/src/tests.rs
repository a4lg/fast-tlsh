// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate`].

#![cfg(test)]

#[cfg(not(fast_tlsh_tests_without_debug_assertions))]
#[test]
fn prerequisites() {
    assert!(
        cfg!(debug_assertions),
        "\
        The tests in this crate requires debug assertions to be enabled (by default).  \
        To test this crate without debug assertions, add rustc flags \"--cfg fast_tlsh_tests_without_debug_assertions\".\
    "
    );
}
