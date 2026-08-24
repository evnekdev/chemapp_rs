//! Source-modelled raw ChemApp ABI types and boundary conversions.
//!
//! This module models the platform preprocessor rules in the checked GTT
//! `cacint.h` revision 2571 source. It deliberately keeps ChemApp `LI`/`LIP`
//! storage separate from Fortran `CHARACTER` lengths: they are not universally
//! the same width. A platform model is not a claim that a corresponding
//! ChemApp binary has been runtime verified.

#[cfg(any(
    all(target_family = "windows", target_pointer_width = "64"),
    all(target_family = "unix", target_arch = "x86_64")
))]
use std::ffi::c_int;
#[cfg(any(
    all(target_family = "windows", not(target_pointer_width = "64")),
    all(target_family = "unix", not(target_arch = "x86_64"))
))]
use std::ffi::c_long;
use std::ffi::CString;
use std::mem::size_of;

use crate::error::ChemAppError;

// `cacint.h`: `_WIN64 || WIN64 || _X86_64 || __x86_64__ || x86_64` selects
// `LI = int` and `LNT = size_t`. Rust's supported 64-bit Windows targets
// define the equivalent Win64 model, including Windows ARM64.
#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
pub(crate) type ChemAppInt = c_int;
#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
pub(crate) type ChemAppLen = usize;

// The other Windows branch in `cacint.h` declares both LI and LNT as `long`.
#[cfg(all(target_family = "windows", not(target_pointer_width = "64")))]
pub(crate) type ChemAppInt = c_long;
#[cfg(all(target_family = "windows", not(target_pointer_width = "64")))]
pub(crate) type ChemAppLen = c_long;

// `cacint.h` selects `LI = int` and `ftnlen = int` only for UNIX/x86-64.
#[cfg(all(target_family = "unix", target_arch = "x86_64"))]
pub(crate) type ChemAppInt = c_int;
#[cfg(all(target_family = "unix", target_arch = "x86_64"))]
pub(crate) type ChemAppLen = c_int;

// The literal non-x86-64 UNIX fallback in the checked source is `long` for
// both LI and ftnlen. This is a source fallback, not binary verification for
// every architecture that reaches it.
#[cfg(all(target_family = "unix", not(target_arch = "x86_64")))]
pub(crate) type ChemAppInt = c_long;
#[cfg(all(target_family = "unix", not(target_arch = "x86_64")))]
pub(crate) type ChemAppLen = c_long;

#[cfg(not(any(target_family = "windows", target_family = "unix")))]
compile_error!("ChemApp's checked ABI model covers Windows and Unix targets only");

// Compile-time checks tie the Rust aliases to the actual C aliases named by
// the source model. They intentionally do not hard-code a generic 4- or
// 8-byte assertion for source-fallback platforms.
#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
const _: [(); size_of::<ChemAppInt>()] = [(); size_of::<c_int>()];
#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
const _: [(); size_of::<ChemAppLen>()] = [(); size_of::<usize>()];

#[cfg(all(target_family = "windows", not(target_pointer_width = "64")))]
const _: [(); size_of::<ChemAppInt>()] = [(); size_of::<c_long>()];
#[cfg(all(target_family = "windows", not(target_pointer_width = "64")))]
const _: [(); size_of::<ChemAppLen>()] = [(); size_of::<c_long>()];

#[cfg(all(target_family = "unix", target_arch = "x86_64"))]
const _: [(); size_of::<ChemAppInt>()] = [(); size_of::<c_int>()];
#[cfg(all(target_family = "unix", target_arch = "x86_64"))]
const _: [(); size_of::<ChemAppLen>()] = [(); size_of::<c_int>()];

#[cfg(all(target_family = "unix", not(target_arch = "x86_64")))]
const _: [(); size_of::<ChemAppInt>()] = [(); size_of::<c_long>()];
#[cfg(all(target_family = "unix", not(target_arch = "x86_64")))]
const _: [(); size_of::<ChemAppLen>()] = [(); size_of::<c_long>()];

/// Converts a public non-negative Rust index/count to raw `LI` without
/// truncation. APIs with native negative selectors must retain `ChemAppInt`
/// until they implement those selectors deliberately.
pub(crate) fn usize_to_chemapp_int(value: usize) -> Result<ChemAppInt, ChemAppError> {
    ChemAppInt::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "ChemApp INTEGER cannot represent Rust value {value}"
        ))
    })
}

/// Converts an existing signed public value before it crosses the raw ABI.
pub(crate) fn i32_to_chemapp_int(value: i32) -> Result<ChemAppInt, ChemAppError> {
    ChemAppInt::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "ChemApp INTEGER cannot represent public i32 value {value}"
        ))
    })
}

/// Converts a native value used as a public count/index. Negative native
/// selectors must not be silently reinterpreted as large `usize` values.
pub(crate) fn chemapp_int_to_usize(value: ChemAppInt) -> Result<usize, ChemAppError> {
    usize::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "negative ChemApp INTEGER {value} cannot be represented as usize"
        ))
    })
}

/// Adapts a raw native integer to the established public `i32` representation
/// without assuming source-fallback `long` storage is always 32-bit.
pub(crate) fn chemapp_int_to_i32(value: ChemAppInt) -> Result<i32, ChemAppError> {
    // This is an identity conversion on checked c_int branches, but remains a
    // checked narrowing on the source-modelled c_long fallback. Keep one
    // portable boundary rather than cfg-splitting every caller.
    #[allow(clippy::useless_conversion)]
    i32::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "ChemApp INTEGER {value} cannot be represented as public i32"
        ))
    })
}

pub(crate) fn chemapp_int_to_u32(value: ChemAppInt) -> Result<u32, ChemAppError> {
    u32::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "ChemApp INTEGER {value} cannot be represented as u32"
        ))
    })
}

/// Converts fixed-size native integer output arrays to the public `i32`
/// fields used by transparent-file metadata.
pub(crate) fn chemapp_int_array_to_i32<const N: usize>(
    values: [ChemAppInt; N],
) -> Result<[i32; N], ChemAppError> {
    let mut converted = [0_i32; N];
    for (destination, value) in converted.iter_mut().zip(values) {
        *destination = chemapp_int_to_i32(value)?;
    }
    Ok(converted)
}

/// Converts an input byte count to the raw target-specific Fortran
/// `CHARACTER` length type. This is deliberately independent from `LI`.
pub(crate) fn usize_to_chemapp_character_length(value: usize) -> Result<ChemAppLen, ChemAppError> {
    ChemAppLen::try_from(value).map_err(|_| {
        ChemAppError::OtherError(format!(
            "Fortran CHARACTER length cannot represent Rust byte length {value}"
        ))
    })
}

/// Returns a valid raw input CHARACTER pointer and its matching declared
/// length. `CString` owns a NUL even for an empty byte sequence, but the NUL
/// is excluded from the Fortran length.
pub(crate) fn cstring_character_input(
    cstring: &CString,
) -> Result<(*const u8, ChemAppLen), ChemAppError> {
    Ok((
        cstring.as_ptr().cast::<u8>(),
        usize_to_chemapp_character_length(cstring.as_bytes().len())?,
    ))
}

/// Converts a native `NOERR` result at the public error boundary.
///
/// `ChemAppError::NativeError` is intentionally `i32` for its established
/// public API. A source-fallback target whose wider raw `long` error code
/// cannot fit is reported as an adaptation failure rather than silently cast.
pub(crate) fn wrap_result<T>(result: T, errcode: ChemAppInt) -> Result<T, ChemAppError> {
    if errcode == 0 {
        Ok(result)
    } else {
        Err(ChemAppError::NativeError(chemapp_int_to_i32(errcode)?))
    }
}

/// Checks the native error before adapting a non-negative ChemApp count or
/// index to Rust, so failed calls never expose an unspecified output value.
pub(crate) fn wrap_nonnegative_result(
    value: ChemAppInt,
    errcode: ChemAppInt,
) -> Result<usize, ChemAppError> {
    wrap_result((), errcode)?;
    chemapp_int_to_usize(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_aliases_follow_the_active_source_model() {
        #[cfg(all(target_family = "windows", target_pointer_width = "64"))]
        {
            assert_eq!(size_of::<ChemAppInt>(), size_of::<c_int>());
            assert_eq!(size_of::<ChemAppLen>(), size_of::<usize>());
        }
        #[cfg(all(target_family = "windows", not(target_pointer_width = "64")))]
        {
            assert_eq!(size_of::<ChemAppInt>(), size_of::<c_long>());
            assert_eq!(size_of::<ChemAppLen>(), size_of::<c_long>());
        }
        #[cfg(all(target_family = "unix", target_arch = "x86_64"))]
        {
            assert_eq!(size_of::<ChemAppInt>(), size_of::<c_int>());
            assert_eq!(size_of::<ChemAppLen>(), size_of::<c_int>());
        }
        #[cfg(all(target_family = "unix", not(target_arch = "x86_64")))]
        {
            assert_eq!(size_of::<ChemAppInt>(), size_of::<c_long>());
            assert_eq!(size_of::<ChemAppLen>(), size_of::<c_long>());
        }
    }

    #[test]
    fn raw_integer_and_character_length_conversions_are_checked() {
        assert_eq!(usize_to_chemapp_int(0).unwrap(), 0);
        assert_eq!(chemapp_int_to_usize(0).unwrap(), 0);
        assert!(chemapp_int_to_usize(ChemAppInt::MIN).is_err());
        assert!(usize_to_chemapp_character_length(0).is_ok());

        if size_of::<ChemAppInt>() < size_of::<usize>() {
            assert!(usize_to_chemapp_int(usize::MAX).is_err());
        }
        if size_of::<ChemAppLen>() < size_of::<usize>() {
            assert!(usize_to_chemapp_character_length(usize::MAX).is_err());
        }
    }

    #[test]
    fn cstring_input_can_represent_an_empty_character_argument() {
        let empty = CString::new("").unwrap();
        let (pointer, length) = cstring_character_input(&empty).unwrap();
        assert_eq!(length, 0);
        assert!(!pointer.is_null());
        assert_eq!(unsafe { *pointer }, 0);
    }

    #[test]
    fn public_error_code_adaptation_is_checked() {
        assert!(matches!(
            wrap_result((), ChemAppInt::MIN),
            Err(ChemAppError::NativeError(_)) | Err(ChemAppError::OtherError(_))
        ));
    }
}
