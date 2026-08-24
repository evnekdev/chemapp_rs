# ChemApp manual knowledge base

This directory is the development knowledge base for keeping `chemapp_rs` aligned with the official ChemApp Programmer's Manual **and** with the raw Fortran ABI that `src/native.rs` dynamically calls.

## Authoritative sources

Primary semantic source:

- ChemApp Online Documentation: https://gtt-technologies.de/software/chemapp/online-manual/
- Direct manual index: https://gtt-technologies.de/ca-doc/index.html

The official documentation defines ChemApp concepts, subroutine behavior, option meanings, state transitions, indexing, units, errors, and recommended programming practice.

The legacy programmer's manual is not a complete machine-level ABI specification. In particular, signatures shown for C or FORTRAN use may hide compiler ABI details such as Fortran CHARACTER lengths.

For every real run, the actual ChemApp version reported by `TQVERS` is authoritative for compatibility decisions. Compatibility-sensitive code should document which ChemApp versions, compilers/builds, architectures, and platforms have actually been exercised.

## Native binding model

Every `src/native.rs` function resolves and calls an exported **Fortran ABI function directly** through `libloading`.

`chemapp_rs` does not call through GTT's C transition layer.

GTT's C interface files are nevertheless critical ABI references:

- `cacint.h` describes the C-facing interface;
- `cacint.c` shows how that C-facing interface is translated into calls to the underlying Fortran routines.

See [Direct Fortran ABI and the ChemApp C transition layer](abi-and-c-interface.md).

## Purpose

This knowledge base exists to answer four questions during development:

1. What does ChemApp define a concept or routine to mean?
2. What preconditions, call ordering, indices, units, option values, side effects, and error semantics does the official API impose?
3. What is the actual Fortran ABI that `Engine` must call for each supported binary?
4. Where may higher-level Rust APIs such as `Calculator`, entities, iterators, snapshots, and caches add ergonomics without changing ChemApp's meaning?

This material is a paraphrased engineering reference. Do not copy substantial portions of the GTT manual into this repository.

## Pages

- [Concepts and state model](concepts.md)
- [Official best practices translated to Rust](best-practices.md)
- [Direct Fortran ABI and the ChemApp C transition layer](abi-and-c-interface.md)
- [Subroutine index](subroutine-index.md)
- [Entities, snapshots, tables, and mapping](entities-and-snapshots.md)
- [Interaction inspection and name resolution](interactions.md)
- [Interaction parameter addressing and reversible mutation](parameter-mutation.md)
- [Development policy](development-policy.md)
- [Current conformance notes](conformance.md)

## Authority order

Do not collapse semantic and ABI authority into one list.

### Semantic behavior

1. Official ChemApp Programmer's Manual.
2. Official ChemApp examples, especially `cademo1.c`, for demonstrated call sequences.
3. Version-matched official documentation for later ChemApp releases where relevant.

### Raw native ABI used by `native.rs`

1. The actual exported DLL/SO ABI of the ChemApp binary being supported.
2. Version/compiler/platform-matched `cacint.c`, because it shows how GTT calls the Fortran routines from C.
3. Matching `cacint.h`, for the public C-side types and declarations.
4. Export inspection and focused runtime tests.

The manual's displayed function signature is **not sufficient evidence** for a Rust `libloading::Symbol<fn(...)>` declaration when ABI details are hidden.

Existing `chemapp_rs` code is always an implementation to be checked, never an authority that overrides ChemApp semantics or the real binary ABI.

If sources disagree, document the discrepancy and keep version-specific handling localized.

## Change workflow

Before adding or changing a native wrapper or high-level operation:

1. Read the complete corresponding manual section for semantics.
2. Record parameter roles, allowed options, index meaning, units, valid call order, state mutation, result lifetime, and errors.
3. Check the equivalent official C example when available.
4. For native FFI, inspect matching `cacint.h` and especially `cacint.c` when available.
5. Reconstruct the raw Fortran ABI, including hidden CHARACTER lengths, argument order, widths, references, calling convention, and symbol decoration.
6. Confirm that the Rust `Symbol<fn>` declaration matches the actual supported DLL/SO export.
7. Add or update tests/examples that exercise the documented behavior.
8. Update this knowledge base when the change reveals a new rule, discrepancy, or version-specific behavior.

The goal is that future implementation starts from documented ChemApp semantics plus verified Fortran ABI behavior rather than reverse-engineering assumptions from existing Rust code.
