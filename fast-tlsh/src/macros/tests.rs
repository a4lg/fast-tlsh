// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::macros`].

#![cfg(test)]

#[forbid(unsafe_code)]
#[cfg(all(test, not(fast_tlsh_tests_without_debug_assertions)))]
#[test]
#[should_panic]
fn violation_invariant() {
    // On tests, an invariant is just a debug_assert,
    // that should work outside an unsafe block.
    super::invariant!(false);
}
