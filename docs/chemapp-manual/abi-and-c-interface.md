# Direct Fortran ABI and the ChemApp C transition layer

This page defines the ABI model used by `chemapp_rs`.

## Hard architectural fact

Every function in `src/native.rs` dynamically resolves and calls a **Fortran ABI function exported by the ChemApp DLL/shared library**.

`chemapp_rs` does **not** link to or call through `cacint.c`, `cacint.h`, `libChemAppC.a`, or another C wrapper layer.

This distinction is fundamental when validating FFI signatures.

## What the official C interface is

GTT supplies C/C++ support using a transition layer whose important source files are:

- `cacint.h` — C-facing typedefs, declarations, macros, and interface definitions;
- `cacint.c` — transition code that adapts normal C calls to the underlying Fortran ABI.

Depending on a ChemApp distribution, an equivalent precompiled C transition library may be supplied instead of compiling `cacint.c` directly.

The C interface is useful to this project because it documents how GTT itself translates between C and the Fortran implementation. It is **reference evidence**, not the ABI actually invoked by `native.rs`.

## Four separate layers must not be conflated

### 1. Programmer's Manual: semantic contract

The manual is the primary source for:

- routine purpose;
- parameter meaning;
- allowed option strings;
- index semantics;
- units;
- valid call order;
- state prerequisites and mutations;
- result meaning;
- documented errors.

The signatures shown in the manual are not sufficient to define the machine ABI.

### 2. `cacint.h`: public C-side contract

The header shows the types and call shape expected by a C/C++ ChemApp application. It is particularly useful for identifying GTT's intended C integer/real types, array shapes, and which values are inputs or outputs.

However, a declaration in `cacint.h` may omit details that exist only between the C transition code and Fortran.

### 3. `cacint.c`: C-to-Fortran translation evidence

The transition source is especially important for ABI reconstruction because it shows how each C-visible call is converted into a Fortran call.

For character arguments this may include **hidden Fortran string-length arguments** that are not visible in the manual's convenient C signature. It may also reveal argument reordering, temporary buffers, blank-padding, integer conversions, and compiler/platform conditionals.

When available, `cacint.c` should be inspected for every ABI-sensitive routine.

### 4. Exported ChemApp DLL/SO ABI: what Rust actually calls

This is the final authority for the FFI declaration used by `libloading`.

For each supported ChemApp binary we must verify, as applicable:

- exported symbol name;
- symbol decoration/mangling;
- calling convention;
- argument order;
- integer width;
- floating-point width;
- by-reference versus by-value passing;
- pointer/array representation;
- hidden character-length parameters;
- placement of hidden character lengths;
- return convention.

`defs.rs` handles exported symbol names, but symbol names alone do not prove the Rust function type is correct.

## Fortran character arguments are a first-class ABI concern

A manual or C-facing call such as a routine taking a character string can be misleading if copied literally into Rust.

Traditional Fortran ABIs commonly pass character data as an address plus one or more hidden length values. The position of those hidden lengths is compiler/platform dependent. They may be appended after explicit arguments or placed differently by a particular interface/compiler convention.

Therefore:

> A ChemApp routine containing one or more character arguments must never be considered ABI-verified from the Programmer's Manual signature alone.

Its implementation must be checked against `cacint.c`/`cacint.h` where available and against the actual exported binary ABI for the supported build.

The existing `native.rs` already reflects this reality. For example, some Windows and Unix bindings use different explicit argument ordering for string lengths. Such differences are ABI adaptations and must remain documented rather than normalized away without evidence.

## Rust wrapper rule

For a `native.rs` function, the Rust method itself may present an ergonomic Rust signature, but the `libloading::Symbol<extern ... fn(...)>` declaration inside that method must match the **raw Fortran ABI exactly** for the target ChemApp binary.

The wrapper may then perform safe adaptation around the call, such as:

- constructing fixed-size or blank-padded character buffers;
- supplying hidden character lengths;
- converting returned character buffers to Rust `String`;
- converting the native error code into `ChemAppError`;
- assembling multiple output arguments into a Rust tuple or struct.

It must not infer ABI details merely from the manual's language-level synopsis.

## Required audit record for each routine

The native conformance matrix should contain separate columns for:

1. routine name;
2. manual semantic signature;
3. `cacint.h` C declaration;
4. relevant `cacint.c` bridge call;
5. raw exported symbol for each supported platform/build;
6. raw Fortran ABI argument list;
7. hidden string lengths and their positions;
8. Rust `Symbol<fn>` declaration;
9. verdict and test evidence.

This separation is mandatory. A routine can be semantically correct but ABI-wrong, or ABI-correct while exposing the wrong high-level semantics.

## Current repository note

`examples/cademo1.c` includes `cacint.h`, but the repository does not currently contain `cacint.h` or `cacint.c` themselves. Until those source files are added to an appropriate reference location, ABI audits must obtain them from the matching ChemApp distribution and record which ChemApp/compiler/platform package they came from.

Do not assume that a `cacint.c` from one ChemApp/compiler build proves the ABI of every other DLL/SO build.