# Configuring this Crate with `--cfg` (Special Options)

This crate accepts some `--cfg` options.

## `fast_tlsh_tests_without_debug_assertions`

By default, test requires debug assertions to be enabled.  By specifying
`--cfg fast_tlsh_tests_without_debug_assertions`, this behavior can be disabled.

## `fast_tlsh_tests_reduce_on_miri`

By specifying `--cfg fast_tlsh_tests_reduce_on_miri`, it reduces the number of
test cases (particularly fuzzing tests) when this crate is being tested with
Miri.
