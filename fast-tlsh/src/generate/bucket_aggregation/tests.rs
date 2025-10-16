// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::generate::bucket_aggregation`].

#![cfg(test)]

use super::naive::{self, get_quartile};
use super::{aggregate_128, aggregate_256, aggregate_48};

use crate::internals::buckets::{NUM_BUCKETS_LONG, NUM_BUCKETS_NORMAL, NUM_BUCKETS_SHORT};
use crate::internals::hash::body::{BODY_SIZE_LONG, BODY_SIZE_NORMAL, BODY_SIZE_SHORT};

#[test]
fn test_naive_get_quartile() {
    // Basic examples
    assert_eq!(get_quartile(0x00, 0x02, 0x04, 0x06), 0b00);
    assert_eq!(get_quartile(0x01, 0x02, 0x04, 0x06), 0b00);
    assert_eq!(get_quartile(0x02, 0x02, 0x04, 0x06), 0b00);
    assert_eq!(get_quartile(0x03, 0x02, 0x04, 0x06), 0b01);
    assert_eq!(get_quartile(0x04, 0x02, 0x04, 0x06), 0b01);
    assert_eq!(get_quartile(0x05, 0x02, 0x04, 0x06), 0b10);
    assert_eq!(get_quartile(0x06, 0x02, 0x04, 0x06), 0b10);
    assert_eq!(get_quartile(0x07, 0x02, 0x04, 0x06), 0b11);
    assert_eq!(get_quartile(0x08, 0x02, 0x04, 0x06), 0b11);
    // Q1 and Q2 are equal
    assert_eq!(get_quartile(0x00, 0x02, 0x02, 0x04), 0b00);
    assert_eq!(get_quartile(0x01, 0x02, 0x02, 0x04), 0b00);
    assert_eq!(get_quartile(0x02, 0x02, 0x02, 0x04), 0b00);
    assert_eq!(get_quartile(0x03, 0x02, 0x02, 0x04), 0b10);
    assert_eq!(get_quartile(0x04, 0x02, 0x02, 0x04), 0b10);
    assert_eq!(get_quartile(0x05, 0x02, 0x02, 0x04), 0b11);
    assert_eq!(get_quartile(0x06, 0x02, 0x02, 0x04), 0b11);
    // Q2 and Q3 are equal
    assert_eq!(get_quartile(0x00, 0x02, 0x04, 0x04), 0b00);
    assert_eq!(get_quartile(0x01, 0x02, 0x04, 0x04), 0b00);
    assert_eq!(get_quartile(0x02, 0x02, 0x04, 0x04), 0b00);
    assert_eq!(get_quartile(0x03, 0x02, 0x04, 0x04), 0b01);
    assert_eq!(get_quartile(0x04, 0x02, 0x04, 0x04), 0b01);
    assert_eq!(get_quartile(0x05, 0x02, 0x04, 0x04), 0b11);
    assert_eq!(get_quartile(0x06, 0x02, 0x04, 0x04), 0b11);
    // Q1, Q2 and Q3 are equal
    assert_eq!(get_quartile(0x00, 0x02, 0x02, 0x02), 0b00);
    assert_eq!(get_quartile(0x01, 0x02, 0x02, 0x02), 0b00);
    assert_eq!(get_quartile(0x02, 0x02, 0x02, 0x02), 0b00);
    assert_eq!(get_quartile(0x03, 0x02, 0x02, 0x02), 0b11);
}

trait BucketAggregationImpls<const SIZE_BODY: usize, const SIZE_BUCKETS: usize> {
    fn naive(out: &mut [u8; SIZE_BODY], buckets: &[u32; SIZE_BUCKETS], q1: u32, q2: u32, q3: u32);
    fn fast(out: &mut [u8; SIZE_BODY], buckets: &[u32; SIZE_BUCKETS], q1: u32, q2: u32, q3: u32);
}

struct BucketAggregation<const SIZE_BODY: usize, const SIZE_BUCKETS: usize>;

impl BucketAggregationImpls<BODY_SIZE_SHORT, NUM_BUCKETS_SHORT>
    for BucketAggregation<BODY_SIZE_SHORT, NUM_BUCKETS_SHORT>
{
    fn naive(
        out: &mut [u8; BODY_SIZE_SHORT],
        buckets: &[u32; NUM_BUCKETS_SHORT],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        naive::aggregate_48(out, buckets, q1, q2, q3)
    }

    fn fast(
        out: &mut [u8; BODY_SIZE_SHORT],
        buckets: &[u32; NUM_BUCKETS_SHORT],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        aggregate_48(out, buckets, q1, q2, q3)
    }
}

impl BucketAggregationImpls<BODY_SIZE_NORMAL, NUM_BUCKETS_NORMAL>
    for BucketAggregation<BODY_SIZE_NORMAL, NUM_BUCKETS_NORMAL>
{
    fn naive(
        out: &mut [u8; BODY_SIZE_NORMAL],
        buckets: &[u32; NUM_BUCKETS_NORMAL],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        naive::aggregate_128(out, buckets, q1, q2, q3)
    }

    fn fast(
        out: &mut [u8; BODY_SIZE_NORMAL],
        buckets: &[u32; NUM_BUCKETS_NORMAL],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        aggregate_128(out, buckets, q1, q2, q3)
    }
}

impl BucketAggregationImpls<BODY_SIZE_LONG, NUM_BUCKETS_LONG>
    for BucketAggregation<BODY_SIZE_LONG, NUM_BUCKETS_LONG>
{
    fn naive(
        out: &mut [u8; BODY_SIZE_LONG],
        buckets: &[u32; NUM_BUCKETS_LONG],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        naive::aggregate_256(out, buckets, q1, q2, q3)
    }

    fn fast(
        out: &mut [u8; BODY_SIZE_LONG],
        buckets: &[u32; NUM_BUCKETS_LONG],
        q1: u32,
        q2: u32,
        q3: u32,
    ) {
        aggregate_256(out, buckets, q1, q2, q3)
    }
}

#[test]
fn equivalence_optimized_impl() {
    fn test<const SIZE_BODY: usize, const SIZE_BUCKETS: usize>()
    where
        BucketAggregation<SIZE_BODY, SIZE_BUCKETS>: BucketAggregationImpls<SIZE_BODY, SIZE_BUCKETS>,
    {
        // Step evaluation
        let mut buckets = [0u32; SIZE_BUCKETS];
        buckets.iter_mut().zip(1u32..).for_each(|(o, i)| *o = i);
        let test = |q1, q2, q3| {
            let mut out1 = [0u8; SIZE_BODY];
            let mut out2 = [0u8; SIZE_BODY];
            BucketAggregation::<SIZE_BODY, SIZE_BUCKETS>::naive(&mut out1, &buckets, q1, q2, q3);
            BucketAggregation::<SIZE_BODY, SIZE_BUCKETS>::fast(&mut out2, &buckets, q1, q2, q3);
            assert_eq!(out1, out2);
        };
        for offset in 0..=SIZE_BUCKETS as u32 * 2 {
            test(offset, offset, offset);
        }
        for offset in 0..=((SIZE_BUCKETS as u32 / 4) * 4) {
            test(offset, offset, offset + (SIZE_BUCKETS as u32 / 4));
            test(
                offset,
                offset + (SIZE_BUCKETS as u32 / 4),
                offset + (SIZE_BUCKETS as u32 / 4),
            );
        }
        for offset in 0..=((SIZE_BUCKETS as u32 / 4) * 3) {
            test(
                offset,
                offset + (SIZE_BUCKETS as u32 / 4),
                offset + (SIZE_BUCKETS as u32 / 4) * 2,
            );
        }
    }
    test::<BODY_SIZE_SHORT, NUM_BUCKETS_SHORT>();
    test::<BODY_SIZE_NORMAL, NUM_BUCKETS_NORMAL>();
    test::<BODY_SIZE_LONG, NUM_BUCKETS_LONG>();
}
