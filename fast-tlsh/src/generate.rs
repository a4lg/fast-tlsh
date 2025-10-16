// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright 2013 Trend Micro Incorporated
// SPDX-FileCopyrightText: Copyright (C) 2024, 2025 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The fuzzy hash generator.

use crate::internals::errors::GeneratorError;
use crate::internals::generate::{GeneratorOptions, GeneratorType};
use crate::internals::params::{ConstrainedFuzzyHashParams, ConstrainedFuzzyHashType};

pub use crate::internals::generate::WINDOW_SIZE;

/// The macro representing the inner generator type.
macro_rules! inner_type {
    ($ty:ty) => {
        <<$ty as ConstrainedFuzzyHashType>::Params as ConstrainedFuzzyHashParams>::InnerGeneratorType
    };
}

/// The fuzzy hash generator corresponding specified fuzzy hash type.
///
/// For the main functionalities, see [`GeneratorType`] documentation.
#[derive(Debug, Clone)]
pub struct Generator<T: ConstrainedFuzzyHashType> {
    /// The inner object representing actual contents of the generator.
    pub(crate) inner:
        <<T as ConstrainedFuzzyHashType>::Params as ConstrainedFuzzyHashParams>::InnerGeneratorType,
}
impl<T: ConstrainedFuzzyHashType> Generator<T> {
    /// Creates the new generator.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}
impl<T: ConstrainedFuzzyHashType> Default for Generator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: ConstrainedFuzzyHashType> GeneratorType for Generator<T> {
    type Output = T;

    const IS_CHECKSUM_EFFECTIVE: bool = <inner_type!(T)>::IS_CHECKSUM_EFFECTIVE;
    const MIN: u32 = <inner_type!(T)>::MIN;
    const MIN_CONSERVATIVE: u32 = <inner_type!(T)>::MIN_CONSERVATIVE;
    const MAX: u32 = <inner_type!(T)>::MAX;

    #[inline(always)]
    fn processed_len(&self) -> Option<u32> {
        self.inner.processed_len()
    }

    #[inline(always)]
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    #[inline(always)]
    fn finalize_with_options(
        &self,
        options: &GeneratorOptions,
    ) -> Result<Self::Output, GeneratorError> {
        self.inner.finalize_with_options(options).map(T::new)
    }

    #[cfg(test)]
    fn count_nonzero_buckets(&self) -> usize {
        self.inner.count_nonzero_buckets()
    }
}

pub(crate) mod tests;
