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

### 1. `Calculator::load_datafile` follows the ChemApp FILE-unit lifecycle — aligned

`Calculator::load_datafile` first validates the case-insensitive `.dat`,
`.cst`, or `.bin` extension, before querying or changing native state. It then
uses `TQGIO("FILE")` to obtain the configured unit, uses that same unit for
the format-specific open/read/close sequence, and attempts `TQCLOS` whenever
the open succeeded.

When the read fails, the close is still attempted. A read error remains the
primary `ChemAppError`; if cleanup also fails, `CleanupError` carries both
native failures. Unsupported filenames are rejected before a `TQGIO` query or
native open call.

This follows the manual's format-specific data-file and close contract without
assuming the default unit 10.

The checked Win64 `entitiesdemo` exercised this path against `data/cosi.dat`:
it initialized the library, queried the configured file unit, loaded and
closed the ASCII data-file, constructed the identity transform from all
component names, and completed a simple equilibrium calculation. The runtime
report retained no license-specific data.

### 2. Calculator constructors propagate initialization failures — aligned

`Calculator::from_library` now follows the explicit sequence `Engine::new` →
`TQINI` → data-file load → fallible component-name collection → fallible
identity `Transform` construction → `Calculator`. `from_library_unloaded`
propagates `Engine::new` and `TQINI` failures and retains the default transform
only because no loaded component basis exists yet.

Recoverable constructor failures, including an unavailable library, now return
`ChemAppError`; focused tests cover both constructor paths without a native
runtime.

`redirect_error_to_temp` likewise avoids `expect`/`unwrap`, falls back to the
normal temporary directory when a library name has no parent directory, and
restores the previous ERROR unit during `Calculator` drop when redirection was
successfully established.

### 3. `TQGETR` scalar subset — aligned and intentionally bounded

`Engine::tqgetr(option, indexp: usize, index: usize) -> Result<f64, _>` is a
complete wrapper for the scalar selector forms that its unsigned public API can
reach: a positive phase and positive individual index; a positive phase with
index zero; `indexp == 0` with a positive system-component index; and the
whole-system `(0, 0)` form. The native result remains valid only for the last
relevant calculation or mapping state and uses ChemApp's active units.

The manual's negative selector forms produce arrays. They cannot be expressed
by this Rust signature, so they cannot reach the one-`f64` native output
buffer. Aggregate result retrieval is deliberately unexposed optional API
surface, not a current scalar ABI/safety defect.

### 4. Live entity errors — corrected

The authoritative live entity accessors and count-dependent iterator
constructors now return `Result`. Native failures are no longer converted to
`NaN`, false, placeholder names, or silently empty sets. Optional diagnostic
or lossy APIs, if added later, must remain explicitly named and separate.

### 5. `mapping_temperature` / `mapping_pressure` — corrected

The high-level mapping helpers now exhaust the documented FIRST/NEXT
continuation sequence, snapshot every successful current result before the
next native call, retain the final result with a non-positive continuation
indicator, and preserve native order. `indexp` and `indexc` are forwarded
independently, and `list` selects `TQMAPL` only when requested.

### 6. Calculation reset/pressure assumptions need documentation — audit

`calculate_isothermal_` calls `TQREMC(-2)`, sets temperature and incoming amounts, then calculates equilibrium. The manual documents default pressure as 1 bar, so this can legitimately rely on ChemApp's default state, but the high-level API currently does not make the assumed pressure obvious.

**Direction:** explicitly document the pressure behavior or provide an API that accepts pressure rather than relying silently on a default.

### 7. Same-`Engine` concurrency semantics are not encoded — audit

ChemApp is stateful. The crate's architecture explicitly anticipates parallelism through separately loaded library instances/copies, but the public type system/documentation does not yet make the non-reentrant nature of a single engine prominent.

**Direction:** document the rule now; later audit auto-traits and consider an explicit synchronization/ownership design if necessary.

### 8. Integer/string ABI verification — audited, platform limits retained

The routine-by-routine audit is complete in
[native-abi-audit.md](native-abi-audit.md). The current raw layer distinguishes
signed 32-bit ChemApp INTEGER storage from platform-specific CHARACTER length
types and models CString inputs as pointers plus checked byte lengths. The
checked Win64 binary and Win32/Linux source evidence are recorded there;
Unix64 remains unverified because no matching binary is checked in.

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
already correct. `TQGETR` is now documented as a verified scalar-result
subset: its unsigned selectors cannot reach the documented negative array
forms, so one `f64` is safe for every exposed call. Win32/x86 therefore has
75 verified wrappers. Direct disassembly of the
checked 2017 Win64 DLL establishes signed 32-bit raw `LI`/`LIP`/`NOERR`
storage and 64-bit non-UNIX `LNT` values. Current master implements those
distinct roles through source-modelled raw `ChemAppInt` and `ChemAppLen`
aliases, with checked public `usize` conversions. This removes the former
common Win64 ABI-ISSUE, but no wrapper is promoted to Win64 VERIFIED
without complete per-routine evidence: all 75 rows remain UNVERIFIED.
Linux/i386 has 68 UNVERIFIED wrappers and 7
absent exports; Unix64 has no checked binary. The subsequent platform-ABI
modelling pass moved raw aliases into `src/abi.rs`: Win64 remains
`LI`/`LIP`/`NOERR = c_int` with `LNT = usize`, while the checked transition
source selects UNIX/x86-64 `LI`/`ftnlen = c_int` and the literal non-x86-64
UNIX fallback `= c_long`. Those source models are not runtime-support claims.
Every input `CString` remains a raw pointer plus its matching checked byte
length.

The checked Win64 `maindemo` run observed a non-empty TQGTNM result containing
internal spaces with no trailing padding. It also executed the canonical
`TQCPRT -> immediate TQERR` sequence and obtained three structurally complete
records. Installation-specific text and identifiers were not retained in
repository documentation or tests. The source-modelled alias refactor repeated
that Win64 run successfully; the generated demo result artifact was removed.

The current Win64 entity conformance run confirms that Species is not limited
to model codes beginning with `SUB`: the `IDMX` gas phase in `cosi.dat` has
one sublattice and 15 captured species, while each of its seven `PURE` phases
has none. The full live report, including the additional Species rows, equals
its snapshot report before the engine advances. Stream creation, snapshotting,
and consuming removal also completed successfully; duplicate native-name
semantics remain undocumented, so the high-level layer rejects duplicate live
owners rather than assuming a replace/share behavior.

The former CRITICAL and HIGH defects above are **FIXED IN CURRENT MASTER**.
The direct-binary Win64 integer and length questions are likewise resolved
for the checked 2017 x64 DLL and implemented at the raw boundary. Unix64
remains unverified. Aggregate `TQGETR` retrieval remains a possible additive
API design, but is not a prerequisite for scalar correctness.

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

### Entity, snapshot, table, and mapping layer — implemented

The authoritative live entity path now propagates native errors, snapshots
own the complete retained hierarchy and active units, and `stable_only` uses
the strict project rule `AC > 0.9999`. Shared `comfy-table` row schemas render
live and immutable state without separate formatting implementations.

High-level mapping now follows the full FIRST/NEXT continuation protocol,
snapshots every successful result before advancing, forwards `indexp` and
`indexc` separately, and selects `TQMAPL` only when listing was requested.
See [entities-and-snapshots.md](entities-and-snapshots.md).

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

## Next recommended milestone

The scalar `TQGETR` wrapper is an intentional, verified subset; aggregate
retrieval is an optional additive API only when a concrete use case requires
it. The next milestone should instead follow actual project needs. Native
TQBOND model conformance remains limited by the absence of a non-proprietary
SUBG/QUAS/QSOL data set.

The routine-by-routine native ABI audit is already complete for the current
75-wrapper surface; platform-specific binary verification remains a separate,
ongoing evidence task.
