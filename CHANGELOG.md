# Change log

## [0.2.0]? -

### Added
  - Support for Unix-like platforms
  - `TransparentHeader` structure
  - Support for magnetic interactions
  - added `from_library_unloaded` to `Calculator`
  - fallible live entities and self-contained calculator/stream snapshots,
    including active units and full phase descendants
  - deterministic shared live/snapshot tables using `comfy-table`
  - model-aware TQBOND pair/quadruplet entities and combinatorial iterators

### Changed
  - `usize` in `tqgthi` output to `i32`
  - `ChemAppError::NativeError` now retains the signed native error type
    (`i32`) rather than `usize`; public positive indices are checked before
    conversion to the native ChemApp integer.
  - `Calculator::from_library` and `from_library_unloaded` now propagate
    library, initialization, component-name, and composition-transform errors
    instead of panicking. `Calculator::set_clim` now returns `Result`.
  - `Engine::tqgetr` is documented as the deliberately scalar-only result
    subset. Negative array selectors remain unexposed rather than being
    incorrectly treated as scalar calls.
  - `Calculator::snapshot` now returns `Result`; snapshot filtering is explicit
    through `SnapshotOptions` and uses the strict `AC > 0.9999` rule.
  - live entity getters and count-dependent iterator constructors now propagate
    `ChemAppError` rather than silently returning placeholders or empty sets.
  - `Engine::tqmap` and `Engine::tqmapl` now return signed `i32` continuation
    values so the documented non-positive terminal states remain representable.
  - `Stream::remove(self)` provides observable consuming native cleanup;
    high-level streams now have unique ownership per stream name.
  - a failed `Calculator::set_clim` retry now reports both bounded-ordering
    failures through `ChemAppError::RetryError`.

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
  - modelled the checked `cacint.h` integer and hidden CHARACTER-length
    branches explicitly: Win32 `long`, Win64 `_WIN64` `int`/`size_t`,
    UNIX/x86-64 `int`, and the literal non-x86-64 UNIX `long` fallback.
  - pass input `CString` values to native CHARACTER parameters as raw pointers,
    including safely representable zero-length inputs.
  - load data-files through the configured `TQGIO("FILE")` unit and always
    attempt `TQCLOS` after a successful open; dual read/close failures preserve
    the primary error and cleanup context.
  - made `redirect_error_to_temp` fallible without `expect`/`unwrap`, including
    a normal temporary-directory fallback for library names with no parent.
  - made the Calculator/entity example project-relative and `Result`-based.
  - completed high-level TQMAP/TQMAPL continuation, corrected listed-routine
    selection and `indexc` forwarding, preserved signed `ICONT`, and snapshot
    every native mapping state.
  - replaced the old quadruplet-only `Bond` concept with explicit QUAS/QSOL
    pair and SUBG quadruplet identities, including canonical enumeration.
  - implemented real stream ownership cleanup through `Drop` and fallible
    stream property/snapshot access.
  - corrected species enumeration to use ChemApp's documented non-mixture
    (`PURE`) versus solution-phase distinction instead of a `SUB*` prefix.
  - corrected `Calculator::set_clim` so either failed call in its preferred
    two-bound order retries the complete reverse ordering once.
  - made examples select only architecture-matched bundled ChemApp binaries,
    with `CHEMAPP_LIBRARY` and optional `CHEMAPP_DATAFILE` overrides.
