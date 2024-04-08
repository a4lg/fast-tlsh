# fast-tlsh: Fast TLSH-compatible Fuzzy Hashing Library in pure Rust

[TLSH](https://tlsh.org/) stands for Trendmicro Locality Sensitive Hash.
TLSH can be used to detect similar files.

You can generate / parse / compare (TLSH-compatible) LSHs with this crate.

Thanks to SIMD-friendly optimizations and its memory layout, comparing two LSHs
are significantly faster than the original implementation.  Even if you turn
off real SIMD (to forbid any unsafe code), it employs pseudo-SIMD operations
and additional tables to speed up the comparison.

Also, it speeds up generating fuzzy hashes (~50% faster) using the
"double update" table optimization.


## Crate Features (Major)

*   `alloc` and `std` (default)  
    This crate supports `no_std` (by disabling both of them) and
    `alloc` and `std` are built on the minimum `no_std` implementation.
    Those features enable implementations that depend on `alloc` and `std`,
    respectively.
*   `easy-functions` (default)  
    It provides easy-to-use high-level functions.
*   `simd` (default; **fast but unsafe**)  
    This crate is unsafe by default (due to the use of SIMD instructions).
    But you can benefit from other optimizations even if you disable it.
*   `detect-features` (default; **marginally slow but convenient**)  
    This feature depends on `std`.  
    If the `simd` feature is enabled and there's the case where switching
    between SIMD and non-SIMD implementations are feasible, it turns on the
    runtime checks to switch the implementation dynamically.
*   `opt-default` (default; Recommended if no default features are enabled)  
    This crate implements number of optimizations and may be tuned separately.
    If you turn off all default features, all such optimizations are turned off.
    You may enable this feature for recommended set of optimizations *except*
    real SIMD-based ones (that are generally unsafe).
*   `opt-embedded-default` (Turn off the default features if you use this)  
    By default, this crate is optimized for cache-rich environment.
    For embedded devices with a smaller cache memory, you may use this feature
    to turn off generating large tables.  It makes the code slightly
    bigger but currently reduces the static memory footprint by 128.25KiB.
*   `strict-parser`  
    It enables the strict parser which enforces additional validity.
    This is disabled by default (because it is not implemented in the official
    implementation) but enabling it will make the parser more robust.
*   `unsafe` (**marginally fast but unsafe**)  
    Other unsafe features not related to SIMD are masked behind the
    default-disabled feature: `unsafe`.  Note that, enabling this feature will
    not (normally) speed up the program significantly.
*   `unstable`  
    This feature enables some features specific to the Nightly Rust *except*
    portable SIMD.  Note that this feature heavily depends on the version of
    `rustc` and should not be considered stable (don't expect SemVer-compatible
    semantics).
*   `serde`  
    It enables integration with [Serde](https://serde.rs/) to serialize /
    deserialize fuzzy hashes.
*   `tests-slow`  
    They will enable "slow" tests (including fuzzing tests).

For all features (including minor tuning-related ones), see the documentation.
