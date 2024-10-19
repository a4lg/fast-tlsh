// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! The easy wrapper for generator functionalities (for `std` environment).

#![cfg(all(feature = "std", feature = "easy-functions"))]

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::errors::GeneratorOrIOError;
use crate::generate::Generator;
use crate::macros::{invariant, optionally_unsafe};
use crate::params::ConstrainedFuzzyHashType;
use crate::{GeneratorType, Tlsh};

/// Constant temporary buffer size for "easy" functions.
const BUFFER_SIZE: usize = 1048576;

/// Generates a fuzzy hash from a given reader stream.
///
/// This is a common function grouping buffering part.
///
/// # Performance Consideration
///
/// It doesn't use [`BufReader`](std::io::BufReader) because the internal buffer
/// is large enough.  Note that the default buffer size of `BufReader` is
/// normally 8KiB (while [buffer size](BUFFER_SIZE) here has 1MiB).
#[inline]
fn hash_stream_common<R: Read, G: GeneratorType>(
    generator: &mut G,
    reader: &mut R,
) -> Result<G::Output, GeneratorOrIOError> {
    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        let len = reader.read(&mut buffer)?;
        if len == 0 {
            break;
        }
        optionally_unsafe! {
            invariant!(len <= buffer.len());
        }
        generator.update(&buffer[0..len]);
    }
    Ok(generator.finalize()?)
}

/// Generates a fuzzy hash from a given reader stream
/// (with specified output type).
///
/// # Example
///
/// ```
/// use std::fs::File;
///
/// type CustomTlsh = tlsh::hashes::Short;
///
/// fn main() -> Result<(), tlsh::GeneratorOrIOError> {
///     let mut stream = File::open("data/examples/smallexe.exe")?;
///     let fuzzy_hash: CustomTlsh = tlsh::hash_stream_for(&mut stream)?;
///     let fuzzy_hash_str = fuzzy_hash.to_string();
///     assert_eq!(fuzzy_hash_str, "T140E0483A5DFC1B073D86A4A2C55A43");
///     Ok(())
/// }
/// ```
pub fn hash_stream_for<T: ConstrainedFuzzyHashType, R: Read>(
    reader: &mut R,
) -> Result<T, GeneratorOrIOError> {
    let mut generator = Generator::<T>::new();
    hash_stream_common(&mut generator, reader)
}

/// Generates a fuzzy hash from a given reader stream.
///
/// # Example
///
/// ```
/// use std::fs::File;
///
/// fn main() -> Result<(), tlsh::GeneratorOrIOError> {
///     let mut stream = File::open("data/examples/smallexe.exe")?;
///     let fuzzy_hash = tlsh::hash_stream(&mut stream)?;
///     let fuzzy_hash_str = fuzzy_hash.to_string();
///     assert_eq!(fuzzy_hash_str, "T1FFE04C037F895471D42E5530499E47473757E5E456D28B13ED1944654C8534C7CE9E01");
///     Ok(())
/// }
/// ```
pub fn hash_stream<R: Read>(reader: &mut R) -> Result<Tlsh, GeneratorOrIOError> {
    hash_stream_for::<Tlsh, _>(reader)
}

/// Generates a fuzzy hash from a given file
/// (with specified output type).
///
/// # Example
///
/// ```
/// type CustomTlsh = tlsh::hashes::Short;
///
/// fn main() -> Result<(), tlsh::GeneratorOrIOError> {
///     let fuzzy_hash: CustomTlsh = tlsh::hash_file_for("data/examples/smallexe.exe")?;
///     let fuzzy_hash_str = fuzzy_hash.to_string();
///     assert_eq!(fuzzy_hash_str, "T140E0483A5DFC1B073D86A4A2C55A43");
///     Ok(())
/// }
/// ```
pub fn hash_file_for<T: ConstrainedFuzzyHashType, P: AsRef<Path>>(
    path: P,
) -> Result<T, GeneratorOrIOError> {
    let mut file = File::open(path)?;
    let mut generator = Generator::new();
    hash_stream_common(&mut generator, &mut file)
}

/// Generates a fuzzy hash from a given file.
///
/// # Example
///
/// ```
/// fn main() -> Result<(), tlsh::GeneratorOrIOError> {
///     let fuzzy_hash = tlsh::hash_file("data/examples/smallexe.exe")?;
///     let fuzzy_hash_str = fuzzy_hash.to_string();
///     assert_eq!(fuzzy_hash_str, "T1FFE04C037F895471D42E5530499E47473757E5E456D28B13ED1944654C8534C7CE9E01");
///     Ok(())
/// }
/// ```
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<Tlsh, GeneratorOrIOError> {
    hash_file_for::<Tlsh, _>(path)
}

mod tests;
