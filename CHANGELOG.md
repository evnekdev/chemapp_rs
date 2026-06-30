# Change log

## [0.2.0]? -

### Added
  - Support for Unix-like platforms
  - `TransparentHeader` structure
  - Support for magnetic interactions
  - added `from_library_unloaded` to `Calculator`

### Changed
  - `usize` in `tqgthi` output to `i32`

### Fixed

  - `tqgthi` function signature
  - changed `_TQERR@4` to `_TQERR@12` for win32 native interface
  - fixed `tqgdat` signature, does not crash anymore
  - missing implementation of the `tqgtrh` function