// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Fuzzy hashes with specific parameters.
//!
//! # What should I use?
//!
//! [`Normal`], as the name suggests.
//!
//! Other configurations are useful on some cases but often lack interoperability
//! due to the lack of large datasets using non-normal variants.
//!
//! To endorse using this variant, this crate re-exports [`Normal`] as
//! crate global [`Tlsh`](crate::Tlsh).
//!
//! # Configuration
//!
//! This crate provides two configurable parameters:
//!
//! 1.  The number of buckets
//! 2.  The length of the checksum
//!
//! ## Number of Buckets
//!
//! Name                                                       | Value | Body     | Official Name | Meaning
//! ---------------------------------------------------------- | -----:| -------- | ------------- | ------------------------------------------------------------
//! [`NUM_BUCKETS_SHORT`](crate::buckets::NUM_BUCKETS_SHORT)   |  `48` | 12 bytes | min hash      | Short, 48 effective buckets (special Pearson table is used)
//! [`NUM_BUCKETS_NORMAL`](crate::buckets::NUM_BUCKETS_NORMAL) | `128` | 32 bytes | compact hash  | Normal, 128 effective buckets
//! [`NUM_BUCKETS_LONG`](crate::buckets::NUM_BUCKETS_LONG)     | `256` | 64 bytes | full hash     | Long, 256 effective buckets
//!
//! ## Length of the Checksum
//!
//! Name                                                                  | Value | Meaning
//! --------------------------------------------------------------------- | -----:| ----------------------------
//! [`CHECKSUM_SIZE_NORMAL`](crate::hash::checksum::CHECKSUM_SIZE_NORMAL) |   `1` | Normal checksum (in 1-byte)
//! [`CHECKSUM_SIZE_LONG`](crate::hash::checksum::CHECKSUM_SIZE_LONG)     |   `3` | Long checksum (in 3-bytes)
//!
//! # Table of Fuzzy Hash Types and Parameters
//!
//! Bucket size \ Checksum size  | Normal: `1` | Long: `3`
//! ----------------------------:|:----------- |:---------------------------
//!       Short (min hash): `48` | [`Short`]   | N/A
//! Normal (compact hash): `128` | [`Normal`]  | [`NormalWithLongChecksum`]
//!      Long (full hash): `256` | [`Long`]    | [`LongWithLongChecksum`]
//!
//! Note that not all parameter combinations are valid.

pub use crate::params::exported_hashes::*;
