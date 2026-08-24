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