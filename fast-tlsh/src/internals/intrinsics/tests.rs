// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::internals::intrinsics`].

#![cfg(test)]

use super::{likely, unlikely};

#[test]
fn test_likely_unlikely() {
    assert!(likely(true));
    assert!(!likely(false));
    assert!(unlikely(true));
    assert!(!unlikely(false));
}
