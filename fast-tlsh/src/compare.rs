// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>

//! Comparison-related metrics and the configuration type.

pub(crate) mod dist_body;
pub(crate) mod dist_checksum;
pub(crate) mod dist_length;
pub(crate) mod dist_qratios;
pub(crate) mod utils;

/// Denotes the mode of comparison (between two fuzzy hashes).
///
/// For description of the parts, see [`FuzzyHashType`](crate::FuzzyHashType).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComparisonConfiguration {
    /// The default mode.
    ///
    /// In this default mode, all checksum, length, Q ratio pair and body
    /// are compared to another.
    #[default]
    Default,
    /// The no-length distance mode.
    ///
    /// In this mode, all checksum, Q ratio pair and body (all *except* the
    /// length encoding) are compared to another.
    ///
    /// # Compatibility Note
    ///
    /// This is renamed from an erroneous name `NoDistance`.
    NoLength,
}

mod tests;
