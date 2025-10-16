// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

macro_rules! fuzz_aggregate_template {
    {$($name:ident = ($method_to_test:ident, $size:literal, $seed:literal, $iter:expr);)*} => {
        $(
            #[test]
            fn $name() {
                assert_eq!($size % 4, 0);
                let mut rng = Xoshiro256PlusPlus::seed_from_u64($seed);
                let mut buckets = [0; $size];
                for _ in 0..$iter {
                    buckets.iter_mut().for_each(|x| *x = rng.random());
                    let mut buckets_sorted = buckets;
                    buckets_sorted.sort_unstable();
                    let q1 = buckets_sorted[$size / 4 * 1 - 1];
                    let q2 = buckets_sorted[$size / 4 * 2 - 1];
                    let q3 = buckets_sorted[$size / 4 * 3 - 1];
                    let mut expected_out = [0; $size / 4];
                    super::naive::$method_to_test(&mut expected_out, &buckets, q1, q2, q3);
                    let mut out = [0; $size / 4];
                    super::$method_to_test(&mut out, &buckets, q1, q2, q3);
                    assert_eq!(
                        out, expected_out,
                        "failed on buckets={buckets:?}, q1={q1}, q2={q2}, q3={q3}"
                    );
                }
            }
        )*
    }
}

#[cfg(all(miri, fast_tlsh_tests_reduce_on_miri))]
const ITER: usize = 1_000;
#[cfg(not(all(miri, fast_tlsh_tests_reduce_on_miri)))]
const ITER: usize = 1_000_000;

fuzz_aggregate_template! {
    fuzz_aggregate_48  = (aggregate_48,   48, 0x381e31a9a5f5714e, ITER);
    fuzz_aggregate_128 = (aggregate_128, 128, 0xcd1476225ecea02c, ITER);
    fuzz_aggregate_256 = (aggregate_256, 256, 0xbd151ebb474984d7, ITER);
}
