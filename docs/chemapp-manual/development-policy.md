# ChemApp-aligned development policy

This file defines hard architectural rules for future `chemapp_rs` development.

The official ChemApp documentation is the semantic authority for native operations:

https://gtt-technologies.de/software/chemapp/online-manual/

The raw Fortran ABI exported by the supported ChemApp DLL/SO is the machine-level authority for every `libloading` call in `src/native.rs`.

## 1. `Engine` is the fidelity layer

`Engine` exists to expose ChemApp's native operations in Rust with the smallest practical semantic change.

A low-level wrapper may:

- convert native error codes to `ChemAppError`;
- convert fixed/blank-padded native strings to Rust strings;
- adapt ABI details that Rust cannot express directly;
- return multiple native output arguments as a tuple/struct;
- use Rust slices/vectors where the native routine returns arrays.

A low-level wrapper must not:

- redefine the physical meaning of an option;
- silently change native index origin;
- silently add calculation/reset calls;
- hide an error by manufacturing a default result;
- alter unit semantics;
- reinterpret a target or mapping operation for convenience.

Convenience belongs above `Engine`.

## 2. Every `native.rs` function calls the Fortran ABI directly

This is a hard architectural fact.

`src/native.rs` does not call GTT's C transition layer. Every `libloading::Symbol<extern ... fn(...)>` declaration must match the **underlying Fortran ABI exported by the ChemApp DLL/SO** for the target build.

The Programmer's Manual is not enough to establish that signature. Its language-level signatures may hide ABI arguments, especially Fortran CHARACTER lengths.

For ABI reconstruction use, in order of relevance:

1. the actual exported ChemApp binary being supported;
2. matching `cacint.c`, which shows how GTT bridges ordinary C calls to the Fortran routines;
3. matching `cacint.h`, which defines the C-facing types/declarations;
4. export inspection and focused runtime tests.

`cacint.h` and `cacint.c` are reference evidence. They are **not** the layer that Rust calls through.

For every routine with character arguments, explicitly verify:

- character buffer representation;
- hidden length arguments;
- hidden-length placement/order;
- blank-padding/trimming expectations;
- differences between Windows and Unix compiler conventions.

No string-taking native wrapper is considered ABI-verified solely because its visible arguments match the manual or `cacint.h` declaration.

See `abi-and-c-interface.md`.

## 3. Every native wrapper needs a documentation trail

When a native wrapper is added or materially changed, its Rust documentation should identify the corresponding ChemApp routine and, where practical, the manual section.

For ABI-sensitive routines, comments must capture platform-specific argument ordering, string-length, integer-width, reference/value, symbol, or calling-convention differences that explain the Rust implementation.

Do not preserve mysterious FFI code solely because it appears to work.

## 4. Native indices stay native

Inside `Engine`, ChemApp indices remain native one-based indices.

Higher-level APIs should prefer typed entities or name-based lookup. If any higher-level API adopts zero-based indexing, that boundary must be explicit in its type/name/documentation and tested carefully.

Never mix zero-based and one-based conventions in a single undocumented parameter.

## 5. Entity identity is name/configuration based, not raw-index based

Long-lived application logic must not treat a raw phase/component/constituent index as stable identity across different data-files or modified systems.

Raw indices may be stored temporarily together with the specific loaded system that produced them for performance, but caches must be invalidated or rebuilt when the thermochemical system or system-component basis changes.

## 6. Errors are data, not defaults

Any native failure must remain observable.

Reusable high-level APIs should normally return `Result`. Methods intended as lossy convenience/diagnostic views must say so explicitly if they substitute values such as `NaN` or `<NONE>`.

New code should not add `unwrap()` to a path where a meaningful `ChemAppError` can be propagated.

## 7. Treat each `Engine` as mutable non-reentrant state

ChemApp maintains internal mutable state inside the loaded native library.

Even if an FFI method takes `&self`, callers must not infer that native operations are logically immutable or safe to execute concurrently on the same engine instance.

Parallel calculations should use independent ChemApp library instances. Any future `Send`/`Sync` exposure must be justified by the actual ChemApp ABI and state model, not by Rust auto-traits alone.

## 8. Calculation helpers must state their reset contract

Every helper that performs an equilibrium calculation must document whether it:

- uses existing conditions;
- removes incoming amounts only;
- resets all calculation conditions while preserving units (`TQREMC(-2)`);
- resets units/configuration as well (`TQREMC(-1)` or `TQINI`);
- changes phase/constituent statuses;
- changes system components;
- changes thermodynamic parameters.

Hidden carry-over between calculations is unacceptable.

## 9. Global-condition and stream workflows remain distinct

Do not mix `TQSETC`-based global input and `TQSTCA`/`TQSTEC` stream input in a single convenience path unless the official ChemApp semantics explicitly permit the sequence.

High-level APIs should make the selected calculation-input model obvious.

## 10. Units must be explicit at API boundaries

ChemApp units are mutable state.

New user-facing functionality must do one of the following:

- explicitly set the units it requires and document that mutation;
- query and return the active unit together with values where ambiguity matters; or
- operate under a higher-level unit policy that is established and verified before calculation.

Do not annotate a returned number with a hard-coded unit merely because that unit is ChemApp's default.

## 11. Use ChemApp's file-I/O contract

Thermochemical files read by ChemApp must be opened/read/closed using the documented ChemApp routines for the format.

The normal input FORTRAN unit should be obtained through `TQGIO("FILE")` rather than hard-coded in production helpers.

Output/error redirection must preserve enough information to restore previous destinations where the `Engine` remains in use afterward.

## 12. Preserve current-state versus snapshot distinction

Live entities (`Phase`, `SystemComponent`, etc.) represent the **current** ChemApp state.

Snapshots represent copied Rust-owned historical state.

Do not make a live entity appear immutable across subsequent calculations. Any API returning a sequence of equilibria or mapping points must materialize each state before advancing ChemApp.

## 13. Capability/version/build differences must be represented explicitly

Do not assume all ChemApp libraries expose the same routine set or ABI.

Before using version-dependent behavior:

- obtain the run-time version with `TQVERS`;
- detect ChemApp light where relevant;
- check symbol availability when a routine is not guaranteed by the minimum supported version;
- record the compiler/platform/build provenance used to establish ABI details;
- document the minimum tested version.

A `cacint.c` from one compiler/platform package does not automatically prove the ABI of another ChemApp binary.

## 14. Option strings should gain typed high-level representations

The low-level string API should remain available for direct ChemApp fidelity.

Higher-level APIs should progressively replace error-prone literal mnemonics with enums/newtypes whose mapping to native strings is exhaustive and documented where the option set is known.

Unknown/new options must still have an escape hatch at the low level so that typed ergonomics do not prevent access to newer ChemApp functionality.

## 15. Every high-level operation should be traceable to native calls

For each nontrivial `Calculator` operation, documentation should make it possible to answer:

- which ChemApp conditions are set;
- which resets occur;
- which target/mapping variables are used;
- which native calculation routine is called;
- which results are subsequently read;
- what engine state is left behind.

This is particularly important for iterative target calculations implemented partly in Rust.

## 16. Native conformance examples are tests of the binding layer

The translated `maindemo.rs`/`cademo1.c` pair should be treated as a conformance asset.

When a native wrapper changes, compare behavior with the canonical ChemApp example where applicable. Prefer adding focused automated tests as the project matures, but retain the broad reference demo because it exercises call sequences that unit tests may miss.

## 17. The knowledge base changes with the code

A change that establishes a new documented ChemApp rule, version difference, ABI discovery, or intentional deviation must update `docs/chemapp-manual/` in the same development task.

Future contributors and AI agents should not have to rediscover important ChemApp semantics or ABI facts from old commits or debugging sessions.

## 18. Do not use Rust pointer-sized integers as a default ABI type

`usize` and `isize` describe Rust pointer width, not necessarily a ChemApp
Fortran/C-transition integer or a hidden CHARACTER length. Raw aliases belong
in `src/abi.rs` and must follow the literal checked `cacint.h` branches:
Win64 (including source-modelled Windows ARM64) uses `c_int` for
`LI`/`LIP`/`NOERR` and `usize` for `LNT`; Win32 uses `c_long` for both; UNIX
x86-64 uses `c_int` for both `LI` and `ftnlen`; and the literal other-UNIX
fallback uses `c_long` for both. A source model is not evidence of a native
binary or runtime support. In particular, do not generalize the checked
Win64 `i32` result or the UNIX/x86-64 `int ftnlen` branch to every target.
Input `CString` CHARACTER arguments must cross the raw boundary as pointers
plus a length from the same C string, never by indexing `as_bytes()[0]` (which
panics for an empty input).
Public `usize` inputs require checked conversion; a native negative selector
must not silently become a large unsigned Rust value. A successful call is
supporting evidence, not a substitute for this target/build-specific record.

The current complete evidence record is [native-abi-audit.md](native-abi-audit.md). It retains the historical critical `TQCHAR` output-type finding, now fixed in current master, and must be consulted before changing a native declaration.

## 19. Snapshot and TQBOND state must remain explicit

Authoritative entity getters, iterator construction, snapshots, and tables
must propagate native errors. Mapping and other multi-state workflows must
snapshot the current result before the next native call. The project-level
stable filter is exactly `AC > 0.9999` and must be evaluated before deep phase
snapshot work.

Never model TQBOND as an unconditional four-member record. Dispatch by TQMODL:
SUBG is a sublattice quadruplet, QUAS/QSOL are phase-constituent pairs, and
other models expose no TQBOND entity. Preserve local sublattice identity and
apply SUBG's combined second-sublattice offset only at the native boundary.

## 20. Do not infer sublattice species from model spelling

`TQMODL` returns `PURE` for a non-mixture phase. `TQNOSL` and `TQNOLC`
describe solution phases as one or more sublattices, so high-level species
enumeration must use the documented non-mixture/solution distinction—not a
`SUB*` prefix or a manually guessed model list. Query failures remain errors;
they are not a signal to silently return no species.

## 21. High-level streams have one native owner per name

The manual defines `TQSTTP` and `TQSTRM` by stream identifier but does not
specify duplicate creation semantics. `Calculator` must therefore lease a
name to at most one live high-level `Stream`; consuming explicit removal
reports native errors and disables best-effort destructor cleanup. Direct
`Engine` calls intentionally remain outside this ergonomic ownership rule.

## 22. Interaction inspection is raw, parsed, then resolved

The exact TQLPAR descriptor and all TQGPAR values remain owned by every
high-level interaction record. Structural parsing and model-aware name
resolution are additive layers, never replacements for the native evidence.
Unknown grammar is a supported diagnostic state, not a reason to panic or
silently discard a row.

Because TQLPAR may corrupt two-digit order text, optional ASCII-DAT recovery
must expose provenance, retain the original native string, and leave live
TQGPAR values authoritative. The recovery provider must establish a
deterministic phase/channel/index mapping by cross-validating healthy rows; it
must not use approximate text matching. The base crate must not require private
parser credentials, and DAT recovery remains inapplicable to BIN/CST unless a
compatible source model is supplied separately.

Parsing identifies the native indexed structure. Resolution separately uses
`TQMODL` and ChemApp metadata to choose between phase constituents and
sublattice-local species. Flattened sublattice namespaces use cumulative
checked ranges and must support more than two sublattices. Interactions belong
to the loaded thermodynamic model, so they are not duplicated into each
equilibrium or mapping snapshot. `ParameterCache` may consume the inspection
layer, but model-specific TQCDAT mutation semantics require separate evidence.
