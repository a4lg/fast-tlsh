// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

#![cfg(all(test, feature = "tests-slow"))]

use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

macro_rules! fuzz_distance_template {
    {$($name:ident = ($method_to_test:ident, $size:literal, $seed:literal, $iter:expr);)*} => {
        $(
            #[test]
            fn $name() {
                let mut rng = Xoshiro256PlusPlus::seed_from_u64($seed);
                let mut body1 = [0; $size];
                let mut body2 = [0; $size];
                for _ in 0..$iter {
                    rng.fill_bytes(body1.as_mut_slice());
                    rng.fill_bytes(body2.as_mut_slice());
                    let expected_score = super::naive::distance(&body1, &body2);
                    assert_eq!(
                        super::pseudo_simd_32::$method_to_test(&body1, &body2),
                        expected_score,
                        "failed on body1={body1:?}, body2={body2:?}"
                    );
                    assert_eq!(
                        super::pseudo_simd_64::$method_to_test(&body1, &body2),
                        expected_score,
                        "failed on body1={body1:?}, body2={body2:?}"
                    );
                    assert_eq!(
                        super::$method_to_test(&body1, &body2),
                        expected_score,
                        "failed on body1={body1:?}, body2={body2:?}"
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

fuzz_distance_template! {
    fuzz_distance_12 = (distance_12, 12, 0x423aa9f2933a29c4, ITER);
    fuzz_distance_32 = (distance_32, 32, 0xf83c14a440b17eba, ITER);
    fuzz_distance_64 = (distance_64, 64, 0xa3cfdcac617a7155, ITER);
}
