# Change log

## [0.2.0]? -

### Added
  - Support for Unix-like platforms
  - `TransparentHeader` structure
  - Support for magnetic interactions
  - added `from_library_unloaded` to `Calculator`

### Changed
  - `usize` in `tqgthi` output to `i32`
  - `ChemAppError::NativeError` now retains the signed native error type
    (`i32`) rather than `usize`; public positive indices are checked before
    conversion to the native ChemApp integer.

### Fixed

  - `tqgthi` function signature
  - changed `_TQERR@4` to `_TQERR@12` for win32 native interface
  - fixed `tqgdat` signature, does not crash anymore
  - missing implementation of the `tqgtrh` function
  - corrected `TQCHAR` output storage from integer to `f64`/ChemApp `DB`.
  - corrected fixed Fortran CHARACTER lengths for `TQGTID`, `TQGTPI`,
    `TQGTHI`, `TQGTRH`, and the per-record `TQERR` length.
  - made the `TQGSPC` writable CHARACTER output mutable, corrected `TQGSU`
    input length handling, and propagated native `TQGPAR` errors.
  - fixed fixed-width output decoding so internal spaces are retained and only
    trailing Fortran padding is removed.
  - corrected Win64 raw `LI`/`LIP`/`NOERR` storage to signed 32-bit values,
    independently of 64-bit Windows CHARACTER lengths.
  - represented UNIX hidden Fortran CHARACTER lengths as checked signed
    32-bit `ftnlen` values according to the checked transition source.
  - pass input `CString` values to native CHARACTER parameters as raw pointers,
    including safely representable zero-length inputs.
