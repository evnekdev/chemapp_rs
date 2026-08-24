# ChemApp manual knowledge base

This directory is the development knowledge base for keeping `chemapp_rs` aligned with the official ChemApp Programmer's Manual.

## Authoritative sources

Primary source:

- ChemApp Online Documentation: https://gtt-technologies.de/software/chemapp/online-manual/
- Direct manual index: https://gtt-technologies.de/ca-doc/index.html

The official online documentation describes itself as the programmer's manual and contains the ChemApp subroutine reference, C and FORTRAN examples, and worked examples.

The legacy manual currently exposed through `ca-doc` is an older programmer's manual. Current ChemApp for Python documentation covers newer ChemApp releases and is useful as a secondary source for discovering later API additions and strongly typed option sets. It must not silently override the semantics of the native ChemApp version actually loaded by `chemapp_rs`.

For every real run, the actual ChemApp version reported by `TQVERS` is authoritative for compatibility decisions. Compatibility-sensitive code should document which ChemApp versions and platforms have actually been exercised.

## Purpose

This knowledge base exists to answer four questions during development:

1. What does ChemApp define a concept or native routine to mean?
2. What preconditions, call ordering, indices, units, option values, side effects, and error semantics does the official API impose?
3. How should `Engine` preserve those semantics at the Rust boundary?
4. Where may higher-level Rust APIs such as `Calculator`, entities, iterators, snapshots, and caches add ergonomics without changing ChemApp's meaning?

This material is a paraphrased engineering reference. Do not copy substantial portions of the GTT manual into this repository.

## Pages

- [Concepts and state model](concepts.md)
- [Official best practices translated to Rust](best-practices.md)
- [Subroutine index](subroutine-index.md)
- [Development policy](development-policy.md)
- [Current conformance notes](conformance.md)

## Authority order

When implementing or reviewing behavior, use this order:

1. Official ChemApp Programmer's Manual for documented semantics.
2. ABI and observed behavior of the actual ChemApp library version in use when compiler/platform/version details differ or the manual is ambiguous.
3. Official ChemApp reference programs, especially `cademo1.c`, for demonstrated call sequences and practical behavior.
4. Current ChemApp for Python documentation as a secondary reference for newer API surface and typed option enumerations.
5. Existing `chemapp_rs` code only as an implementation to be checked, never as an authority that overrides ChemApp semantics.

If sources disagree, document the discrepancy and keep version-specific handling localized.

## Change workflow

Before adding or changing a native wrapper or high-level operation:

1. Locate the corresponding manual section and read the complete routine description.
2. Record parameter roles, allowed options, index meaning, units, valid call order, state mutation, result lifetime, and errors.
3. Check the equivalent official C example when available.
4. Confirm the Rust FFI signature and platform-specific symbol/calling convention against the actual library ABI.
5. Add or update tests/examples that exercise the documented behavior.
6. Update this knowledge base when the change reveals a new rule, discrepancy, or version-specific behavior.

The goal is that future implementation starts from documented ChemApp behavior rather than reverse-engineering assumptions from existing Rust code.