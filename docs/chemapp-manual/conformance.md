# Current ChemApp conformance notes

This is a living audit of the current `master` implementation against documented ChemApp behavior. The low-level inventory has now received a routine-by-routine audit; see [native ABI audit](native-abi-audit.md). It is scoped to the checked-in binaries and records remaining build-specific unknowns rather than certifying every ChemApp release.

Status vocabulary:

- **aligned** — current design clearly follows the documented concept;
- **audit** — plausible implementation, but signature/semantics need explicit verification;
- **gap** — known divergence, incomplete behavior, or missing robustness;
- **experimental** — intentionally incomplete advanced functionality.

## Architectural alignment

### Dynamic native loading — aligned

`Engine` owns a `libloading::Library`, allowing the ChemApp library path to be chosen at run time. This fits the project's need to support multiple platform/library variants and independent loaded copies.

### Platform-specific symbol aliases — aligned in concept, ABI audit required

`defs.rs` maintains mappings for Win32, Win64, Unix32, and Unix64 symbol naming. This is the correct architectural location for compiler/platform export-name differences.

However, symbol name correctness is only half of ABI conformance. Each routine's integer width, calling convention, argument order, pointer/array semantics, and hidden string-length arguments still need a systematic verification pass against the actual libraries we support.

### Native error conversion — aligned in concept

The core `Engine` wrappers convert nonzero native error codes into `ChemAppError::NativeError` rather than silently continuing. `error.rs` also contains human-readable descriptions for the legacy ChemApp error table.

### `Engine` versus `Calculator` layering — aligned

The native `tq...` operations live in `Engine`; workflow behavior such as data loading, composition transforms, target helpers, parameter caching, and mapping helpers lives above it.

### Entity/iterator model — aligned

Components, phases, constituents, species, and bonds are enumerated from run-time counts and represented as live views over a `Calculator`. This is substantially safer than pervasive raw hard-coded indices.

### Snapshot model — aligned and important

`CalculatorSnapshot` copies the current state into Rust-owned structures. This correctly recognizes that live entity values are tied to the mutable current ChemApp result state.

## High-priority gaps and audit items

### 1. `Calculator::load_datafile` hard-wires FORTRAN unit 10 — gap

The current high-level loader uses unit `10` directly for `.dat`, `.cst`, and `.bin` files.

The ChemApp manual recommends obtaining the data-file unit through `TQGIO("FILE")` before opening a thermochemical data-file. Unit 10 is the documented default, so the current implementation will usually work, but it ignores a mutable ChemApp I/O configuration.

**Direction:** query `TQGIO("FILE")` and use the returned unit in high-level loading.

### 2. High-level constructors contain `unwrap()` despite returning `Result` — gap

`Calculator::from_library` and related paths use `unwrap()` for operations such as `Engine::new` and composition-transform construction even though the public constructor itself returns `Result`.

This can turn an ordinary load/configuration failure into a Rust panic rather than a recoverable error.

**Direction:** propagate or convert all recoverable initialization failures.

### 3. Live entity accessors often suppress native errors — gap

For example, current `Phase` accessors use patterns such as `unwrap_or("<NONE>")` and `unwrap_or(f64::NAN)` for native queries.

This conflicts with the ChemApp manual's strongest best-practice recommendation: check every native error because an apparently later failure may have originated in an earlier call.

**Direction:** establish a consistent API split, for example fallible `Result` accessors as the primary API and optional explicitly named lossy/debug helpers if desired.

### 4. `mapping_temperature` / `mapping_pressure` do not exhaust mapping results — gap

The manual defines one-dimensional phase mapping as a multi-result operation where successive `TQMAP`/`TQMAPL` calls are required until the continuation indicator reports no more results.

The current high-level mapping helpers perform only a fixed small number of calls rather than looping until completion. They therefore cannot be assumed to return every phase transition in an interval.

There is also a closure parameter named `indexc` that is currently ignored in favor of passing `indexp` twice. It is harmless in the current calls because both are zero, but is unsafe if the helper is generalized.

**Direction:** redesign mapping as a proper iterator/state machine or exhaust it in a loop, snapshotting after every result before advancing.

### 5. Calculation reset/pressure assumptions need documentation — audit

`calculate_isothermal_` calls `TQREMC(-2)`, sets temperature and incoming amounts, then calculates equilibrium. The manual documents default pressure as 1 bar, so this can legitimately rely on ChemApp's default state, but the high-level API currently does not make the assumed pressure obvious.

**Direction:** explicitly document the pressure behavior or provide an API that accepts pressure rather than relying silently on a default.

### 6. Same-`Engine` concurrency semantics are not encoded — audit

ChemApp is stateful. The crate's architecture explicitly anticipates parallelism through separately loaded library instances/copies, but the public type system/documentation does not yet make the non-reentrant nature of a single engine prominent.

**Direction:** document the rule now; later audit auto-traits and consider an explicit synchronization/ownership design if necessary.

### 7. Integer/string ABI requires systematic verification — audit

The current native layer contains a mixture of `usize`, `i32`, fixed `u8` buffers, and platform-specific ordering of string-length arguments. Some signatures have already required historical fixes (for example `tqgthi`, `tqgdat`, and `tqerr` according to the changelog).

**Direction:** perform a routine-by-routine ABI audit against native headers/example interfaces and exported symbols for each supported library build. Record the result in a machine-readable or tabular matrix.

## Native ABI audit summary (2026-08-24)

The hardened audit covers all 75 `src/native.rs` wrappers and reports status
per build rather than a misleading cross-platform primary verdict. The first
production correction milestone fixed the nine confirmed Win32/x86 findings:
`TQCHAR` now uses `f64`/`double` storage; fixed CHARACTER calls use their
documented bridge lengths; `TQGSPC` declares its writable pointer as mutable;
`TQGSU` uses the actual input length; and `TQGPAR` propagates native errors.
A follow-up fixed-output conversion correction removed obsolete first-space
decoding from `TQGTNM`, `TQGNSC`, `TQGNP`, `TQMODL`, `TQGNPC`, `TQGNLC`, and
`TQGSP`. In particular, `TQGTNM` now preserves complete multi-word
license-holder text and removes only trailing Fortran padding; its raw ABI was
already correct. Win32/x86 therefore has 74 verified wrappers and one
incomplete API (`TQGETR`) after this correction. Direct disassembly of the
checked 2017 Win64 DLL now establishes 32-bit raw `LI`/`LIP`/`NOERR` storage
and 64-bit non-UNIX `LNT` values. Because current Rust declares every Win64
`NOERR` as `&mut usize`, all 75 Win64 rows now have a confirmed common
ABI-ISSUE; no wrapper is promoted to Win64 VERIFIED by that result.
Linux/i386 has 68 UNVERIFIED wrappers and 7 absent exports; Unix64 has no
checked binary.

The checked Win64 `maindemo` run observed a non-empty TQGTNM result containing
internal spaces with no trailing padding. It also executed the canonical
`TQCPRT -> immediate TQERR` sequence and obtained three structurally complete
records. Installation-specific text and identifiers were not retained in
repository documentation or tests.

The former CRITICAL and HIGH defects above are **FIXED IN CURRENT MASTER** for
the source rules supported by the checked Win32 bridge. This does not resolve
the Win64 `LI`/`LIP`/hidden-length-width question. The next production
milestone is a systematic Win64 raw-integer conversion: use explicit `i32`
raw storage and checked public `usize` conversions while retaining `usize`
for proven 64-bit `LNT` values. The `TQGETR` scalar/array redesign remains
separate.

The checked Win32, Win64, and Linux/i386 exports were inspected. On Win64,
`movl` reads/writes through representative index, count, and error pointers
resolve the `LI`/`LIP` question independently of the successful demo. The
full matrix, explicit character analysis, Unix return-convention conclusion,
exact binary evidence, and C/Rust demo coverage are in
[native-abi-audit.md](native-abi-audit.md).

## Advanced functionality

### Parameter cache — experimental

`cache` is explicitly described in source as newer and subject to change. Gibbs interaction caching/modification is partially implemented; magnetic interactions and several compound/endmember paths contain `todo!()` or commented-out functionality.

The manual also limits thermodynamic data modification to appropriate data-file formats/models. This layer should validate those prerequisites rather than assuming arbitrary loaded systems are mutable.

### Interaction parsing — experimental

Gibbs interaction text parsing is implemented for several forms, but magnetic interaction conversion still contains `todo!()` paths.

Parsing ChemApp's human-readable interaction output is inherently version/model sensitive. Tests should be based on representative official/known data-files and preserve the original unparsed string when interpretation fails.

### Output formatting — incomplete

Some entity formatting helpers such as `Phase::print_header` / `print_values` remain `todo!()`. This is not a native conformance problem by itself, but these methods should not be considered stable API until completed.

## Test/conformance infrastructure

### `cademo1.c` + `maindemo.rs` — valuable alignment asset

The repository includes the canonical-style C demo and a broad Rust translation. This should become the backbone of native wrapper verification.

The Rust demo already resolves project-relative DLL/SO and `cosi.dat` paths, which is preferable to workstation-specific paths.

### Automated native tests — gap

The repository currently lacks a systematic automated conformance suite for the native wrapper layer.

A useful progression is:

1. smoke tests: load library, `TQINI`, version, data-file load;
2. identity tests: component/phase/constituent round-trips;
3. condition tests: set/remove/reset and `TQSHOW`-equivalent state checks;
4. equilibrium tests against known `cosi.dat` reference results;
5. target and mapping tests;
6. stream tests;
7. sublattice/model-specific tests;
8. transparent-file tests where licensing makes them reproducible;
9. data-manipulation tests on ASCII data copies.

Tests should skip with an explicit capability reason when a required full/licensed ChemApp feature is unavailable; they should not silently pass.

## Version gap: legacy manual versus current ChemApp

The legacy online Programmer's Manual documents the API historically used by this crate. Current official ChemApp for Python documentation targets newer ChemApp 8.x releases and exposes additional typed option sets and native capabilities.

We should maintain two separate questions:

1. **Does `chemapp_rs` faithfully wrap the legacy/native surface it already claims to support?**
2. **Which newer native ChemApp capabilities should be added, and from what minimum version?**

Do not mix these into one compatibility assumption.

## Next recommended audit

The next focused task should be a **routine-by-routine audit of `src/native.rs` and `src/defs.rs`**. For each `tq...` function, create a row containing:

- official routine name/manual section;
- supported-since version if documented;
- C signature;
- Rust signature;
- Windows 32/64 and Unix symbol names;
- integer widths;
- string lengths and hidden length arguments;
- option domain;
- index semantics;
- unit semantics;
- state prerequisites/side effects;
- known test/example;
- verdict (`verified`, `fix`, `version-specific`, `untested`).

Only after that matrix is complete should we describe the native layer as fully conformance-audited.
