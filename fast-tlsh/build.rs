// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

fn main() {
    // Avoid unnecessary rebuilding.
    println!("cargo:rerun-if-changed=build.rs");

    // Other cfgs (rustc-check-cfg)
    println!("cargo:rustc-check-cfg=cfg(fast_tlsh_tests_without_debug_assertions)");
    println!("cargo:rustc-check-cfg=cfg(fast_tlsh_tests_reduce_on_miri)");
}
