# AGENTS.md

This file defines repository-wide instructions for AI coding agents and future automated work on `chemapp_rs`.

## Branch and repository policy

- **Always work directly on `master`.**
- Do not create feature branches or pull requests unless the user explicitly overrides this rule for a specific task.
- Before modifying the repository, inspect the current `origin/master` state. Do not assume the local or previously observed commit is still current.
- If `master` advances while work is in progress, reconcile against the new tip before writing. Preserve both legitimate histories; do not silently overwrite concurrent user changes.
- Do not force-push or rewrite published history unless the user explicitly requests it.
- Keep commits focused and descriptive.

## Project purpose

`chemapp_rs` is an unofficial Rust interface to GTT-Technologies ChemApp.

ChemApp itself remains the thermodynamic calculation engine. This crate provides:

1. a low-level dynamically loaded binding to ChemApp's exported **Fortran ABI**;
2. Rust error/string/result adaptation around that ABI;
3. higher-level calculation, entity, iterator, snapshot, parsing, and parameter-cache APIs.

Do not redesign the low-level binding as though `chemapp_rs` were reimplementing ChemApp thermodynamics.

## Mandatory source-of-truth hierarchy

Keep **semantic behavior** and **machine ABI** separate.

### ChemApp semantics

Use, in this order:

1. the official ChemApp Programmer's Manual: https://gtt-technologies.de/software/chemapp/online-manual/;
2. official/reference examples such as `examples/cademo1.c`;
3. version-matched official documentation for later ChemApp releases when relevant.

The manual defines concepts, option meanings, indices, units, valid call order, state changes, result meaning, and documented errors.

### Raw ABI used by `src/native.rs`

Use, in this order:

1. the actual DLL/SO ABI of the ChemApp binary being supported;
2. the matching GTT C-transition implementation in `examples/cacint.c`;
3. the matching C declarations/types in `examples/cacint.h`;
4. exported-symbol inspection and focused runtime/conformance tests.

The manual's visible function signature is **not** sufficient evidence for a Rust FFI function type.

Read `docs/chemapp-manual/README.md`, `docs/chemapp-manual/abi-and-c-interface.md`, and `docs/chemapp-manual/development-policy.md` before substantial native-interface work.

## Hard native-interface rule

Every native wrapper in `src/native.rs` dynamically resolves and calls a **Fortran ABI function directly**.

`native.rs` does not call through `cacint.c`, `cacint.h`, or a C wrapper library. Those files are reference evidence showing how GTT bridges C calls to the Fortran implementation.

For every `libloading::Symbol<extern ... fn(...)>` declaration, verify the raw ABI for the relevant target/build, including:

- exported symbol name and decoration;
- calling convention;
- argument order;
- integer and floating-point widths;
- by-value versus by-reference passing;
- arrays/pointers;
- return convention;
- every hidden Fortran CHARACTER-length argument;
- the placement/order/type of hidden string lengths.

Do not infer or normalize an ABI difference because another platform happens to use a different convention.

## Fortran CHARACTER arguments

String-taking functions require special care.

A C-facing declaration may omit Fortran CHARACTER-length parameters that are present in the raw ABI. `examples/cacint.c` demonstrates that the UNIX and non-UNIX interfaces can place these lengths differently.

Therefore:

- never mark a string-taking wrapper ABI-verified from the manual alone;
- do not copy a `cacint.h` prototype verbatim into Rust and assume it is the exported Fortran signature;
- verify hidden lengths, argument positions, padding, buffer sizes, and output trimming;
- document platform/compiler differences next to the Rust FFI declaration when they are non-obvious.

## `Engine` is the fidelity layer

`Engine` should preserve ChemApp's native meaning with the smallest practical Rust adaptation.

It may:

- convert a native error code into `ChemAppError`;
- adapt fixed-width/blank-padded character buffers;
- supply hidden ABI arguments;
- package multiple output parameters into tuples or structs;
- use safe Rust containers around native arrays when semantics remain unchanged.

It must not:

- redefine option meanings;
- silently change ChemApp's native index origin;
- insert hidden equilibrium/reset operations for convenience;
- discard a nonzero ChemApp error;
- silently manufacture default values after native failure;
- change unit semantics;
- reinterpret target or mapping behavior.

Higher-level ergonomics belong in `Calculator` and layers above it.

## Indices, state, and concurrency

- ChemApp native indices are one-based; preserve that convention inside `Engine`.
- Do not use raw numeric phase/component/constituent indices as persistent identity across different loaded systems.
- ChemApp is stateful. Treat each loaded `Engine` as mutable, non-reentrant native state even when a Rust method takes `&self`.
- Do not introduce concurrent calls on the same `Engine` without explicit ABI/thread-safety evidence.
- Parallel calculation designs should use independent ChemApp library instances/copies unless a verified supported mechanism says otherwise.
- Document reset/state-carryover behavior for every high-level calculation helper.

## Errors

The official ChemApp guidance strongly favors checking every native error close to the call that produced it.

- Native wrappers must preserve every nonzero error code.
- Prefer `Result` propagation in reusable high-level APIs.
- Avoid `unwrap()` in recoverable library paths.
- Do not convert native errors into `NaN`, zero, empty strings, or placeholder names unless an API is explicitly documented as lossy/diagnostic.

## Units and file I/O

- ChemApp units are mutable engine state. Never attach hard-coded unit labels to returned values unless the active unit is established.
- Use ChemApp's documented data-file routines for ChemApp-managed files.
- Obtain configurable ChemApp I/O units through the relevant ChemApp calls rather than assuming defaults in production helpers.
- Ensure successful open/read flows are paired with appropriate closing/cleanup behavior.

## Live entities and snapshots

- Live entities (`Phase`, `SystemComponent`, `Constituent`, etc.) reflect the **current** native ChemApp state.
- Snapshots are Rust-owned copies of a specific calculated state.
- Any mapping/scan API that advances ChemApp through multiple equilibria must snapshot each result before advancing if previous states need to be retained.

## Reference C files

`examples/cacint.c` and `examples/cacint.h` are GTT reference material and contain their own copyright/reuse notice.

- Preserve their copyright and provenance text.
- Treat them as reference inputs, not project-owned code to casually refactor or reformat.
- Do not copy substantial portions of them into Rust documentation.
- Do not assume the checked-in 2013/revision-2499 interface source proves the ABI of every ChemApp version/compiler build.

Likewise, paraphrase the official ChemApp manual in project documentation instead of reproducing substantial manual text.

## Documentation standard

Keep documentation detailed enough that future maintainers do not need to rediscover ABI decisions experimentally.

- Use `//!` for meaningful module-level documentation.
- Use `///` for public API items and for private functions/types whose contract is non-obvious or safety/ABI relevant.
- Add ordinary `//` comments for important private fields, invariants, unusual native constants, ABI adaptations, and state assumptions.
- For every ABI-sensitive wrapper, document why its signature is correct, especially where Windows/Unix differ.
- Do not leave unexplained FFI code merely because it currently works.

When a change establishes a new ChemApp semantic rule, ABI discovery, version difference, or intentional deviation, update `docs/chemapp-manual/` in the same task.

## Testing and conformance

Treat `examples/cademo1.c` and `examples/maindemo.rs` as important binding-conformance assets.

For native-interface changes, prefer evidence in this progression:

1. source-level comparison with the manual and matching `cacint` bridge;
2. exported-symbol/ABI inspection for the target binary;
3. focused wrapper tests;
4. comparison with the canonical/reference C call sequence;
5. broader calculation regression tests.

Run normal Rust quality checks when the environment permits, including formatting and compilation/tests relevant to the changed code. Native ChemApp execution may depend on platform-specific binaries, bitness, licensing, or a hardware/software license mechanism; distinguish an unavailable native capability from a Rust regression and report it explicitly.

Do not claim ABI conformance solely because code compiles.

## High-level API changes

Before adding a nontrivial `Calculator` operation, be able to state:

- which native conditions are set;
- which reset operations occur;
- which phase/constituent statuses may change;
- which native equilibrium/target/mapping routines are called;
- what units are assumed or established;
- which results are read;
- what ChemApp state remains after return;
- how native errors propagate.

Prefer typed high-level enums/newtypes for documented option sets, while retaining low-level escape hatches that preserve access to native ChemApp options.

## Completion checklist

Before considering a task complete:

- confirm work is on current `master`;
- ensure concurrent user changes were preserved;
- verify native semantics against the manual when relevant;
- verify raw FFI details against actual ABI evidence when relevant;
- update `docs/chemapp-manual/` for new knowledge;
- add/update tests or conformance examples where practical;
- keep code and documentation internally consistent;
- report any unverified platform/version assumptions explicitly.