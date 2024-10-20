# All Features and Feature Selection Guide

# Index

*   [All Features](self#all-features)
    *   [Baseline and Runtime](self#baseline-and-runtime)
    *   [User-facing Features](self#user-facing-features)
    *   [Performance / Tuning](self#performance--tuning)
    *   [Testing / Maintenance](self#testing--maintenance)
*   [Guide: Selecting Features for Your Needs](self#guide-selecting-features-for-your-needs)

# All Features

## Baseline and Runtime

*   `alloc` and `std` (default; `std` depends on `alloc`)  
    This crate supports `no_std` (by disabling both of them) and
    `alloc` and `std` are built on the minimum `no_std` implementation.
    Those features enable implementations that depend on `alloc` and `std`,
    respectively.

## User-facing Features

*   `easy-functions` (default)  
    It provides easy-to-use high-level functions.
*   `strict-parser`  
    It enables the strict parser which enforces additional validity.
    This is disabled by default (because it is not implemented in the official
    implementation) but enabling it will make the parser more robust.
*   `serde`  
    It enables integration with [Serde](https://serde.rs/) to serialize /
    deserialize fuzzy hashes.
    *   `serde-buffered` (depends on `serde`)  
        By default, this crate prefers deserialization without any additional
        allocation.  Normally, you don't have to enable this.
        But enabling this feature may improve robustness on certain formats.
        For instance, CBOR deserialization using Ciborium with `serde-buffered`
        makes possible to decode indefinite-length bytes with multiple chunks.
*   `experiment-pearson`  
    Enabling it exposes the internal Pearson hashing and TLSH B (bucket) mapping
    functions.  It may help users fiddling with TLSH internals but otherwise
    not needed.

## More Unsafe Features

*   `unsafe` (unsafe)  
    Other unsafe features that are not related to SIMD are masked behind the
    default-disabled feature: `unsafe`.  Note that, enabling this feature will
    not (normally) speed up the program significantly.
*   `unstable` (unstable)  
    This feature enables some features specific to the Nightly Rust *except*
    portable SIMD.  Note that this feature heavily depends on the version of
    `rustc` and should not be considered stable (don't expect SemVer-compatible
    semantics).

## Performance / Tuning

Not just enabling / disabling major optimizations, you can tune the internals
in detail for your needs.

*   `opt-default` (default)  
    This is a combination of following features for rich environment
    (features below are described later in this section):
    *   `opt-dist-length-table`
    *   `opt-dist-qratios-table-double`
    *   `opt-pearson-table-double`
*   `opt-embedded-default`  
    This is a combination of following features for low cache memory environment
    (features below are described later in this section):
    *   `opt-dist-length-table`
    *   `opt-dist-qratios-table`
    *   `opt-low-memory-hex-str-encode-half-table`
*   `simd` (default; unsafe)  
    This is a combination of following features:
    *   `opt-simd` (safe in this crate but see below)  
        Enables SIMD implementation on following components
        (each can be turned on or off by separately using features below):
        *   `opt-simd-body-comparison` (safe by itself)  
            On comparing fuzzy hashes, use *real* SIMD optimization to compare
            two bodies.  Note that, the fallback when this is disabled is the
            pseudo-SIMD implementation which does the similar using scalars.
        *   `opt-simd-bucket-aggregation` (safe by itself)  
            On generating fuzzy hashes, use SIMD optimization to convert the
            content of the buckets to quartile-based dibit values
            (`0b00..=0b11`).
            Despite that this is particularly fast, it is only effective on
            the finalization.  So, this is not a big optimization when you
            mainly process large files.
        *   `opt-simd-parse-hex` (uses unsafe external crate)  
            By using external `hex-simd` crate, it speeds up parsing the TLSH's
            hexadecimal representation.
        *   `opt-simd-convert-hex` (uses unsafe external crate)  
            By using external `hex-simd` crate, it speeds up conversion from the
            fuzzy hash object to the corresponding hexadecimal representation
            (`to_string()` or serialization to a human-readable format).
    *   `simd-per-arch` (unsafe)  
        It enables architecture-specific SIMD intrinsics-based implementation.
        The actual implementation is available (may be partially supported,
        depending on the component) on following architectures:
        *   `x86` (SSE2 / SSSE3 / SSE4.1 / AVX2)
        *   `x86_64` (SSE2 (baseline) / SSSE3 / SSE4.1 / AVX2)
        *   `wasm32` (128-bit SIMD)
        *   `aarch64` (ASIMD, originally called NEON)
        *   `arm` (NEON; only when the `unstable` feature is enabled and the
            target is ARMv7 or later)
*   `simd-portable` (unstable, safe by itself; depends on `unstable`)
    If you enable this feature, all SIMD implementations but hexadecimal string
    conversion are implemented through (now unstable) [`core::simd`] library.
    This is safe unlike stable `simd-per-arch` implementations but requires
    the Nightly channel of the Rust compiler.
    If the `simd-per-arch` feature is enabled, it will be fallback implementions
    to the architecture-specific ones.
*   `detect-features`
    (default, typically unsafe when in effect; depends on `std`)  
    If the `simd` feature (unsafe) is enabled and there's the case where
    switching between SIMD and non-SIMD implementations are feasible, it turns
    on the runtime checks to switch the implementation dynamically.
    Dynamic switching is currently supported on:
    *   `x86`
    *   `x86_64`
    *   `arm` (only when the `unstable` feature is enabled)
*   `opt-dist-length-table`
    (default via `opt-default`, part of `opt-embedded-default`)  
    Enabling it creates a 512-byte table (2-byte per entry, 256-entry) used when
    comparing encoded length parts of given fuzzy hashes.
*   `opt-dist-qratios-table` (part of `opt-embedded-default`)  
    Enabling it creates a 256-byte table (1-byte per entry, 16*16-entry) used
    when comparing Q ratio values of given fuzzy hashes.
    This is effectively disabled if `opt-dist-qratios-table-double` is enabled.
*   `opt-dist-qratios-table-double` (default via `opt-default`)  
    Enabling it creates a 64KiB table (1-byte per entry, 256*256-entry) used
    when comparing Q ratio pair parts of given fuzzy hashes.
    It computes the distance without extracting low/high 4-bits (Q1 ratio / Q2
    ratio values) unlike in `opt-dist-qratios-table`.
*   `opt-pearson-table-double` (default via `opt-default`)  
    This is the biggest contributor for generator speedups.
    Enabling it creates a 64KiB table based on the existing 256-byte Pearson
    hashing table (always required).  TLSH often use Pearson hashing to 4-byte
    data and some combinations of byte 2 and 3 (in 1-origin) can be shared.
    This optimization utilizes these facts and merges two lookups for byte 2 and
    byte 3 to one lookup ("double update" table optimization).  Not only it
    reduces the lookup, it reduces the cost of address calculation.
*   `opt-low-memory-buckets`  
    By default, the generator has 256 buckets regardless of the variant (48, 128
    or 256 buckets) to reduce branches.  By using this option, it reduces the
    number of buckets to the final one to be used (i.e. halves the bucket
    memory consumption on the normal 128 bucket variant).
*   `opt-low-memory-hex-str-decode-half-table`  
    By default, the embedded hexadecimal digits decoder creates two 512-byte
    tables (each 2-byte per entry, 256-entry; 1KiB in total).  Enabling this
    feature reduces 2 tables to 1 (reducing the table size from 1KiB to
    512 bytes).
*   `opt-low-memory-hex-str-decode-quarter-table`  
    This is a superset of the `opt-low-memory-hex-str-decode-half-table`
    feature.  Not only reducing the number of tables, it halves the size of an
    element in the table (reducing the table size to 256 bytes).
*   `opt-low-memory-hex-str-encode-half-table` (part of `opt-embedded-default`)  
    By default, the embedded hexadecimal digits encoder creates two 512-byte
    tables (each 2-byte per entry, 256-entry; 1KiB in total).  Enabling this
    feature reduces 2 tables to 1 (reducing the table size from 1KiB to
    512 bytes).
*   `opt-low-memory-hex-str-encode-min-table`  
    This is a superset of the `opt-low-memory-hex-str-encode-half-table`
    feature.  It makes the static table for byte to hex digits conversion
    bare minimum of 16 bytes.

## Testing / Maintenance

*   `tests-slow`  
    They will enable "slow" tests (including fuzzing tests).
*   `maint-code`  
    By default, compiler warnings and Clippy warnings are not an error.
    Enabling this will make them error.
*   `maint-lints`  
    By default, a special mitigation to minimize lint-related issues and
    version incompatibilities are enabled.  This feature disables this.


# Guide: Selecting Features for Your Needs

## Default

The default feature set is composed of following features:

*   `std`
*   `easy-functions`
*   `opt-default`
*   `simd`
*   `detect-features`

The intent of those default is, to be fast enough (unsafety is carefully
controlled) and yet convenient to use it as a library.

## `no_std` + Other Optimizations

You may turn off the default features and enable following features:

*   `opt-default`

Of course, you can re-enable `simd` for SIMD-based optimizations.

## `no_std` + Embedded Environment Recommendation

You may turn off the default features and enable following feature:

*   `opt-embedded-default`

Of course, you can re-enable `simd` for SIMD-based optimizations.  Note that
you cannot switch the implementation dynamically because `detect-features`
depends on `std`.

## Safe SIMD

If you use the Nightly Rust and use SIMD safely, turn off the default features
and enable following:

*   `opt-default`
*   `opt-simd`
*   `simd-portable`

Note that it doesn't disable external `hex-simd` crate using unsafe features.
If you want to turn this off, you may use following features instead.

*   `opt-default`
*   **`opt-simd-body-comparison`**
*   **`opt-simd-bucket-aggregation`**
*   `simd-portable`

It disables external `hex-simd` create by disabling `opt-simd-parse-hex` and
`opt-simd-convert-hex` features.

Note that, the result of portable SIMD-based optimization heavily depends on
the code generator and architecture internals.  Depending on various parameters,
you may not get enough boost on safe SIMD (at least, it is confirmed that this
is reasonably fast on recent x86 CPUs).

## The Fastest Configuration (Normally)

If you run your program only on your machine, you may use compiler options
`-C target-cpu=native` and turn off the default-enabled `detect-features`.

If you are going with it, disable the default options and enable those:

*   `opt-default`
*   `simd`
*   `unsafe`
*   `unstable`  
    (only when you are working with the Nightly compiler)
*   `simd-portable`  
    (when you are working with niche architectures; requires Nightly)
