# Changelog

## [1.0.0] - Unreleased

First stable release of the current direct-Fortran binding and high-level API.
ChemApp itself remains a separately obtained proprietary dependency.

### Added
  - Support for Unix-like platforms
  - `TransparentHeader` structure
  - Support for magnetic interactions
  - added `from_library_unloaded` to `Calculator`
  - fallible live entities and self-contained calculator/stream snapshots,
    including active units and full phase descendants
  - deterministic shared live/snapshot tables using `comfy-table`
  - model-aware TQBOND pair/quadruplet entities and combinatorial iterators
  - typed raw → parsed → name-resolved Gibbs and magnetic interaction
    inspection through `Phase` and `Calculator`, with diagnostic tables
  - an optional, dependency-neutral ASCII-DAT interaction cross-check boundary
    with explicit native, disagreement, and validated-recovery provenance
  - typed one-based `InteractionParameterAddress` values and checked
    `Calculator` read/write access for runtime-verified TQGPAR/TQCDAT cells
  - a model-neutral interaction parameter cache retaining complete
    multi-expression Gibbs and SUBLM magnetic matrices, read-only support
    status, structural keys, baseline-relative deltas, and verified resets
  - beginner quickstart, equilibrium, snapshot, interaction, and reversible
    parameter-mutation examples driven by `CHEMAPP_LIBRARY` and
    `CHEMAPP_DATAFILE`
  - standard `std::error::Error` integration for `ChemAppError`

### Changed
  - `usize` in `tqgthi` output to `i32`
  - `ChemAppError::NativeError` now retains the signed native error type
    (`i32`) rather than `usize`; public positive indices are checked before
    conversion to the native ChemApp integer.
  - `Engine` is deliberately `!Sync` because its shared-reference methods
    mutate one native ChemApp state. Sequential ownership transfer remains
    possible; parallel work requires independent supported library instances.
  - removed unused public calculation counters and made Calculator's data-file,
    transform, error-redirection, and parameter-cache storage private, with
    read-only accessors for the first, transform, and cache.
  - composition transforms now report dimension and non-spanning-basis errors
    instead of allowing dependency assertions to unwind through Calculator.
  - added `Calculator::calculate_isothermal_at_pressure`; the existing method
    retains ChemApp's documented 1-bar default after its full condition reset.
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
    values through checked raw-integer conversion, so documented non-positive
    terminal states remain representable on every source-modelled target.
  - `Stream::remove(self)` provides observable consuming native cleanup;
    high-level streams now have unique ownership per stream name.
  - a failed `Calculator::set_clim` retry now reports both bounded-ordering
    failures through `ChemAppError::RetryError`.
  - renamed the interaction recovery surface to a cross-check API and retained
    native parsing, cross-check status, effective structure, and resolution as
    separate provenance layers; deprecated recovery-name aliases forward to the
    same implementation.
  - the interaction cache no longer exposes the former MQM-only
    `InteractionGEMQM`/`InteractionMagnMQM` and six-term, first-expression
    mutation methods; callers use typed structural parameter addresses.
  - `SystemDimensions` fields now use the exact NA–NK dimension meanings rather
    than historical ambiguous names; notably NI is documented as the number of
    Gibbs-energy/heat-capacity equations per constituent and TQGPAR's leading
    dimension.
  - the crates.io package now uses registry `chemformula` 0.2 and an explicit
    distribution allowlist. Proprietary ChemApp DLL/SO files, GTT C reference
    sources, and thermodynamic data-files remain in the development repository
    where applicable but are excluded from the published crate.
  - the table dependency is pinned to its Rust-1.85-compatible 7.1.4 release so
    the declared minimum supported Rust version is reproducible.

### Removed

  - fallible `Engine::default()` and `Calculator::default()` implementations
    which panicked when the external ChemApp library could not be loaded; use
    `Engine::new`, `Calculator::from_library`, or
    `Calculator::from_library_unloaded`.
  - the unimplemented `Calculator::calculate_target_x_from_left` placeholder,
    whose public method body panicked with `todo!()`.

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
  - made the interaction examples portable and fallible, with
    `CHEMAPP_INTERACTION_DATAFILE` support and structured raw/resolved tables.
  - corrected `TQLPAR` output adaptation to honor each returned descriptor
    length and corrected `TQGPAR` multi-expression values from native Fortran
    column-major storage into logical Rust rows.
  - replaced the partial interaction parser path with typed, model-aware
    parsing and native-metadata name resolution; unknown syntax is retained
    explicitly rather than dropped.
  - detect valid-looking structural differences through optional DAT
    cross-checking and recover ChemApp's known multi-digit-order TQLPAR text
    corruption only through its typed validation rule, without replacing live
    TQGPAR values.
  - corrected transformed interaction formatting to omit native parameter and
    arity markers plus diagnostic index annotations while retaining them in
    the structured/raw representations.
  - made sublattice count explicit in interaction reports and descriptors;
    colon-separated groups now preserve variable-sublattice models including
    four-sublattice Olivine interactions.
  - stopped optional DAT-provider errors and ordinary cross-source differences
    from invalidating healthy native interactions; DAT structure now becomes
    effective only for a typed validated native-defect recovery, including the
    known two-digit-order TQLPAR corruption.
  - prevented generic `Unparsed` native interaction text from being labelled a
    ChemApp defect or automatically replaced by a valid DAT-side descriptor.
  - sized TQLPAR records from TQSIZE `ND`/`NE` by channel and TQGPAR's Fortran
    leading dimension from TQSIZE `NI`, retaining the checked-build 28-column
    second extent explicitly.
  - made explicit `Stream::remove` consume destructor cleanup responsibility
    before TQSTRM, so a failed removal cannot trigger a second hidden Drop call.
  - removed unchecked first-element indexing from compound/endmember TQGDAT
    cache loading and report empty native results as contextual errors.
