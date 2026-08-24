# ChemApp best practices translated to Rust

Primary reference: ChemApp Programmer's Manual, section 1.10, "ChemApp best practices":

https://gtt-technologies.de/ca-doc/index.html

These are project rules derived from GTT's own support/development recommendations.

## 1. Log the ChemApp version

The manual recommends recording the run-time ChemApp version returned by `TQVERS` in program output and support/debug material.

### `chemapp_rs` rule

Any reproducible calculation record, diagnostic report, or benchmark should be able to identify the ChemApp version that produced it. Tests that depend on native behavior should record or assert an expected version range where appropriate.

Do not infer compatibility solely from the DLL/SO filename.

## 2. Verify which ChemApp variant is loaded

ChemApp distributions may contain light, standard, extended, optimized, or otherwise specialized builds with different capabilities or maximum system sizes.

### `chemapp_rs` rule

Use `TQLITE`, `TQSIZE`, and version information when capability matters. High-level APIs must not claim support for target calculations or mapping without accounting for the loaded library's capability.

## 3. Check every ChemApp error

The manual is unusually explicit about this: the native error result should be checked after every ChemApp call, because an error observed during equilibrium calculation may actually originate from an earlier lookup or setup operation.

### `chemapp_rs` rule

The low-level `Engine` API must return `Result<_, ChemAppError>` for fallible native calls and must not discard a nonzero ChemApp error code.

Higher-level APIs should propagate or deliberately handle those errors. Avoid converting a failed native query into apparently valid data such as `0`, an empty string, or `NaN` unless the API is explicitly documented as lossy/diagnostic.

Avoid `unwrap()` inside reusable library APIs when the failure can be represented by `ChemAppError`.

## 4. Use ChemApp routines for ChemApp data-file I/O

The manual recommends the format-specific open routines:

- `TQOPNA` for ASCII
- `TQOPNB` for binary
- `TQOPNT` for transparent

and `TQCLOS` for closing.

### `chemapp_rs` rule

High-level loading should use these routines rather than ordinary Rust file handles for files that ChemApp itself must read. The normal input unit should be queried with `TQGIO("FILE")` rather than hard-wired unless a test deliberately exercises a fixed unit. Calculator construction validates a supported extension before querying native state, then uses that configured unit consistently for the format-specific open/read/close sequence. The loader is intentionally private so callers cannot replace the Engine system without rebuilding Calculator metadata.

Any temporary `LIST`/`ERROR` redirection must use valid ChemApp/FORTRAN units, avoid collisions, and restore state when necessary.

## 5. Verify that the expected data-file exists and was loaded

A surprisingly large class of failures comes from loading the wrong file or wrong path.

### `chemapp_rs` rule

High-level constructors should surface path/load failures early and preserve the resolved data-file identity for diagnostics. Tests and examples should prefer paths derived from `CARGO_MANIFEST_DIR` rather than workstation-specific absolute paths.

## 6. Close data-files after reading

The manual warns that failing to close a data-file can cause later reads to begin from the wrong file position.

### `chemapp_rs` rule

Every successful open/read sequence must have a matching `TQCLOS`, including error paths where practical. Calculator's internal loader attempts the close after a failed read as well and retains both failures when native read and close both fail. Where future refactoring makes this feasible, use a small guard abstraction so unit/file cleanup is not dependent on manually repeated code.

## 7. For transparent files, preserve licensing/header diagnostics

The manual recommends reporting the ChemApp user ID and, after successful transparent-file loading, considering the transparent header information.

### `chemapp_rs` rule

When `.cst` loading fails with authorization or expiry-related errors, diagnostics should make `TQGTID`/`TQGTNM` and `TQGTRH`-related information easy to obtain without exposing it unnecessarily in ordinary calculation output.

## 8. Do not hard-wire entity indices

GTT explicitly recommends resolving system component, phase, and constituent indices from names at run time.

### `chemapp_rs` rule

Raw numeric indices are valid only when they are:

- immediately derived from the currently loaded thermochemical system;
- part of a short internal loop over a queried count; or
- deliberately exercising native indexing in a conformance test/example.

Persistent application identity must use names or a higher-level identity object, not `phase == 3`-style assumptions.

## 9. Archive enough information to reproduce a calculation

The manual recommends preserving results, source/build information, the thermochemical data-file, and the ChemApp library version/build used.

### `chemapp_rs` rule

A reproducibility record should be able to identify at minimum:

- `chemapp_rs` revision/version;
- ChemApp version and library identity;
- data-file identity/hash or archived file;
- active units;
- changed phase/constituent statuses;
- input conditions/streams;
- calculation type and target/mapping parameters.

## 10. Use full ChemApp result tables during development/debugging

The manual recommends `TQCEL`, `TQMAPL`, and `TQCENL` output when inspecting suspicious calculations because a set of selected `TQGETR` values can hide important context.

### `chemapp_rs` rule

Keep low-level access to result-table-producing routines. High-level APIs must not remove the ability to obtain a native ChemApp diagnostic table.

## 11. Keep track of incoming amounts and conditions

Iterative/process calculations can accidentally carry unintended input from earlier calculations.

### `chemapp_rs` rule

Calculation helpers must define their reset semantics explicitly. A routine that intends a fresh calculation should use a documented reset path such as `TQREMC(-2)` and then set the required conditions.

For complex workflows, make the final set of conditions inspectable before execution.

## 12. Never assume active units

ChemApp's units are mutable and apply to both inputs and results.

### `chemapp_rs` rule

Do not hard-code units in user-facing output unless the code explicitly set and owns those units. Query them through `TQGSU` or maintain a validated high-level unit policy.

## 13. Make failed equilibrium calculations reproducible

GTT recommends capturing `TQSHOW` before the calculation and preserving it if `TQCE`/`TQCEL` fails, along with result-table output when available.

### `chemapp_rs` rule

A future diagnostic/debug mode should be able to:

1. allocate a unique log target;
2. redirect `LIST` and `ERROR` output;
3. call `TQSHOW` before the equilibrium call;
4. run the calculation using a table-producing variant when appropriate;
5. retain the log only on error (or according to a requested policy);
6. restore previous I/O destinations.

This should eventually be a reusable library facility rather than duplicated application code.

## Additional Rust-specific best practice: typed option values

The original native API uses short string mnemonics such as `T`, `IA`, `AC`, `FILE`, and `ENTERED`. Current official ChemApp for Python documentation replaces many of these strings with enumerations specifically to reduce hard-coded option mistakes.

### `chemapp_rs` direction

Preserve string-compatible native methods for fidelity, but consider typed enums/newtypes in higher-level APIs for:

- condition variables;
- target variables;
- result variables;
- phase-map variables;
- units;
- statuses;
- I/O channels;
- configuration options.

Typed wrappers must have a transparent, documented mapping to the native ChemApp mnemonic and must not invent semantics absent from the native API.

## Snapshot before advancing state

ChemApp result getters describe the current native calculation only. Any
workflow that will calculate again—especially `TQMAP`/`TQMAPL` continuation—
must create an owned snapshot before the next native call. Use
`SnapshotOptions::stable_only()` only when deliberately retaining phases with
`AC > 0.9999`; do not substitute phase amount for that project-level rule.

TQBOND requires model dispatch before enumeration: `SUBG` is a quadruplet,
`QUAS`/`QSOL` are pairs, and other models are not applicable. Keep local
sublattice identity separate from SUBG's combined native index encoding.

## 15. Keep high-level stream ownership singular

ChemApp creates and removes streams by their identifier. The inspected manual
does not define duplicate `TQSTTP` identifier behavior, so a high-level
`Calculator` must not create two independently owning `Stream` values for the
same name. Use one live owner, `Stream::remove(self)` when cleanup errors must
be observed, and regard `Drop` as a best-effort fallback. An owned
`StreamSnapshot` remains valid after stream removal.

Consume destructor responsibility before an explicit native removal call. If
that call fails, report its error and do not let `Drop` issue a second hidden
TQSTRM call; the native state is unknown and the name remains reserved.

## 16. Preserve native interaction evidence before interpretation

`TQLPAR` text is authoritative evidence of what the native library returned;
`TQGPAR` is authoritative for numerical values in the live Engine. Keep both
the exact descriptor and coefficient matrix even when typed parsing or
model-aware name resolution succeeds. Query Gibbs and magnetic channels
independently, use `TQMODL` to choose the native index namespace, and resolve
names through `TQGNPC` or `TQGNLC` rather than parsing a thermodynamic data
file. Unknown syntax must remain visible as unparsed or unresolved data; it
must never disappear from a report.

TQLPAR itself has a known textual defect for multi-digit interaction orders:
valid-looking `[*]` output (or another incomplete fragment) may replace the
actual numeric order. When a compatible ASCII DAT source is available, compare
it deterministically by phase/channel/parameter index and retain the DAT result
as independent evidence. Provider absence or failure must not invalidate a
healthy native row. A difference remains visible while native structure stays
effective unless a typed, validated native-defect rule authorizes recovery.
For the known defect, every other structural field must agree and at least one
native wildcard must correspond to a DAT numeric order of 10 or greater; a
wildcard alone proves nothing. Never guess from a corrupted fragment, and never
replace live TQGPAR values with DAT coefficients.

Treat the TQGPAR buffer as a Fortran array: reconstruct logical expression
rows using its column-major layout and the TQSIZE `NI` leading dimension. Do
not assume that a Rust nested array's row-major indexing has the same meaning.

## 17. Mutate interaction parameters through typed native identity

Use phase/channel/interaction/expression/term-or-role as the physical identity.
Never use formatted TQLPAR text as the mutation key. Ordinary Gibbs values map
to `TQCDAT(13, interaction, expression, term, phase, value)`; the two magnetic
roles map to I1=10 and I4=1/2 respectively. Only expose high-level writes for
model/channel families verified by mutate/readback/restore evidence.

Cache every logical TQGPAR cell, including read-only special columns. Define a
delta relative to the captured baseline, not current live state. Reset by the
stored typed addresses and verify live TQGPAR readback. A TQCDAT write makes
prior equilibrium results stale but must not silently recalculate or clear
conditions.
