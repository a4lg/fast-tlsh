// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

extern crate version_check as rustc;

fn main() {
    // Avoid unnecessary rebuilding.
    println!("cargo:rerun-if-changed=build.rs");

    // Module: core::error
    // unstable: 1.65-1.80 (not implemented)
    //   stable: 1.81-
    println!(
        "cargo:rustc-check-cfg=cfg(\
            fast_tlsh_error_in_core, \
            values(\
                \"stable\"\
            )\
        )"
    );
    if rustc::is_min_version("1.81.0").unwrap_or(false) {
        println!("cargo:rustc-cfg=fast_tlsh_error_in_core=\"stable\"");
    }

    // Other cfgs (rustc-check-cfg)
    println!("cargo:rustc-check-cfg=cfg(fast_tlsh_tests_without_debug_assertions)");
    println!("cargo:rustc-check-cfg=cfg(fast_tlsh_tests_reduce_on_miri)");
}
