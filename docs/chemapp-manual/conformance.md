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

### Platform-specific symbol aliases — aligned for the recorded builds

`defs.rs` maintains mappings for Win32, Win64, Unix32, and Unix64 symbol naming. This is the correct architectural location for compiler/platform export-name differences.

Symbol spelling is only one part of ABI conformance. The systematic audit now
records each routine's calling convention, argument order, numeric storage,
pointer/array semantics, and hidden CHARACTER lengths for the checked binaries.
Unrepresented builds remain explicitly unverified rather than inheriting a
platform-family conclusion.

### Native error conversion — aligned in concept

The core `Engine` wrappers convert nonzero native error codes into `ChemAppError::NativeError` rather than silently continuing. `error.rs` also contains human-readable descriptions for the legacy ChemApp error table.

### `Engine` versus `Calculator` layering — aligned

The native `tq...` operations live in `Engine`; workflow behavior such as data loading, composition transforms, target helpers, parameter caching, and mapping helpers lives above it.

### Entity/iterator model — aligned

Components, phases, constituents, species, and bonds are enumerated from run-time counts and represented as live views over a `Calculator`. This is substantially safer than pervasive raw hard-coded indices.

### Snapshot model — aligned and important

`CalculatorSnapshot` copies the current state into Rust-owned structures. This correctly recognizes that live entity values are tied to the mutable current ChemApp result state.

## High-priority gaps and audit items

### 1. Calculator's internal data-file loader follows the FILE-unit lifecycle — aligned

Calculator construction first validates the case-insensitive `.dat`,
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

The unloaded constructor is intentionally public for interface/version,
licensing, and other operations whose native contract does not require a
thermodynamic system. It is not a reload path: a raw data-file read through
`engine()` cannot update the Calculator's data-file identity, transform,
parameter cache, or system-local high-level metadata. Loaded high-level work
requires constructing a new Calculator with `from_library`.

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

### 6. Calculation reset/pressure contract — resolved for 1.0

The isothermal path calls `TQREMC(-2)`, optionally sets explicit pressure,
sets temperature and transformed incoming system-component mole amounts, then
calls no-target TQCE. The manual documents the post-reset default pressure as
1 bar. `calculate_isothermal_at_pressure` makes pressure explicit;
`calculate_isothermal` deliberately uses that documented default. Both leave
the resulting equilibrium as the current live Engine state.

`calculate_target_t` has a different contract: it does not call `TQREMC` and
therefore inherits the current pressure, units, phase/constituent statuses,
and other active conditions. It sets the phase-amount target, temperature
limits, and incoming system-component amounts before calling temperature-
target `TQCE`; success leaves that target state live, while failure performs no
hidden rollback. Optional fixed/adjusting selectors must be supplied together
as distinct one-based indices within the transformed composition. The Rust
correction now solves `IA_a / IA_f = XP_a / XP_f` in logarithmic ratio space.
The native phase ratio is the first predictor; bounded secant exploration is
used before a sign bracket, followed by a safeguarded bracketed secant/
bisection hybrid. Success tests the physical log-ratio residual, exact zero is
handled separately, and native/non-finite failures propagate. The 32-equilibrium
budget now produces explicit non-convergence instead of the former false
success. See [target-calculations.md](target-calculations.md).

The scalar driver has native-independent tests for residual signs across major
and trace scales, converged and contractive mappings, a divergent legacy Picard
mapping, secant and bracket safeguards, bounded log steps, exact zero, invalid
fractions, native-error propagation, degenerate secants, and explicit budget
failure. The checked Win64 `maindemo` continues to exercise ChemApp's native
target calculation, but the repository has no scientifically defined
fixed/adjusting high-level case; no artificial database-specific case was
invented for this release.

One `Engine` is structurally `!Send + !Sync`: methods taking `&self` still
mutate one native ChemApp state, and no checked source provides a positive
thread-migration contract. Parallel calculations require a build/licence-
supported isolation strategy; repeated loading of one path does not prove
independent native state.

### 7. Same-`Engine` concurrency semantics — encoded for 1.0

ChemApp is stateful. A private zero-sized `PhantomData<Rc<()>>` marker makes
`Engine` and its high-level owners `!Send + !Sync`; compile-fail rustdocs guard
both invariants. The official manual, checked C interface/example sources, and
repository runtime evidence contain no positive statement permitting one
initialized state to move between OS threads. GTT's [public parallel-processing
guidance](https://gtt-technologies.de/2024/01/chemapp-for-python-speeding-up-calculations-by-parallel-processing/)
describes ChemApp as single-threaded and uses independent processes,
which supports the conservative 1.0 contract but is not itself a complete ABI
thread-affinity specification.

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

### Parameter cache — stable API with deliberately bounded coverage

`ParameterCache` is part of the 1.0 API. Its interaction surface is
model-neutral and complete for every TQGPAR cell returned by EN22. It retained
9,034 cells, with 4,042 typed, runtime-verified TQCDAT addresses and 4,992
inspectable read-only SUBQ extended columns. Structural keys include phase,
channel, interaction, expression, and column/role; display text is not physical
identity. Gibbs and SUBLM magnetic absolute/delta/reset paths are implemented,
and delta is always relative to the captured baseline.

The disposable Win64 audit changed/read/restored all 4,042 supported cells,
covered all eight observed matrix shapes, verified six neighboring-interaction
families, confirmed documented Slag-liq copy propagation, and completed six
representative equilibrium mutation/restoration smokes. A separate active-phase
probe changed system Gibbs energy and reproduced its baseline after exact
coefficient restoration. It never called TQWASC and the source DAT digest was
unchanged. Generic SUBQ selectors for
terms 7 and 18 were rejected with error 1024 without changing the matrix, so
columns 7–18 remain read-only. See
[Interaction parameter addressing and reversible mutation](parameter-mutation.md).

Duplicate phase copies remain phase-local addressable views, not independent
parameter owners: the documented TQCDAT write propagates to every copy. No
generic native copy-family identity was found, so the API deliberately does not
infer aliases from `#1`/`#2` display-name suffixes.

The API is stable; model/channel mutation coverage is extensible. Unsupported
or not-yet-verified cells stay inspectable and explicitly read-only rather than
being assigned guessed selectors.

For 1.0, `ParameterCache` exposes only read-only inspection. Cache construction
and every mutation/reset operation are Calculator-owned and always use that
Calculator's Engine, preventing a system-A cache from being applied to a
system-B Engine through safe public API. The Engine field is private;
`engine()` remains the documented advanced low-level accessor. No high-level
reload operation exists yet because construction-only loading is the smallest
coherent contract.

### Interaction inspection — runtime-observed for the EN22 model set

The authoritative path preserves every raw TQLPAR descriptor and TQGPAR
coefficient matrix before parsing or name resolution. Typed parsing and
model-aware resolution cover the runtime-observed `SUBQ`, `SUBL`, `SUBLM`,
`QKTO`, and `QKTOM` grammars, including magnetic `SUBLM`. The EN22 run produced
657 Gibbs and 35 magnetic rows; all 692 were retained, syntactically parsed,
and name-resolved, with zero silently discarded. Cross-validation against the
ASCII DAT semantic model found 25 known TQLPAR multi-digit-order corruptions:
20 SUBQ/G and 5 QKTO/G rows printed `[*]` instead of orders 10–15. The other
667 structures matched by phase/channel/position. The optional, dependency-free
cross-check boundary now retains native parsing separately and reports
`NotRequested`, `Unavailable`, `DatError`, `Agree`, `Disagree`, or a typed
`ValidatedRecovery`. DAT absence, parser failure, and unexplained disagreement
leave healthy native structure effective and cannot erase a row or make its
native resolution fail. Only the validated wildcard-versus-order-10-or-greater
defect (or another explicit recovery class) selects DAT structure. Live TQGPAR
values remain authoritative. Unknown future syntax remains explicitly unparsed.
See [Interaction inspection and name resolution](interactions.md).

The transformed interaction surface now treats each colon-delimited group as
one sublattice and reports the `TQNOSL` phase count explicitly. The native
`*N` interaction arity is not a sublattice count. Runtime coverage includes
one-sublattice Monoxide, two-sublattice Slag/Spinel, and four-sublattice
Olivine descriptors; variable-sublattice models retain every group.

That run also exposed and corrected TQGPAR's Rust-side matrix adaptation:
native Fortran column-major storage is reconstructed as logical
`NOEXPR × NVALA` rows. TQLPAR now honors each returned `LGTPAR` record length.
The hardened Win64 rerun retained/resolved 692 native-only rows and produced
667 agreements, 25 validated recoveries (20 SUBQ/G, 5 QKTO/G), zero ordinary
disagreements, zero DAT errors/unavailable rows, and zero unresolved rows.

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

The interaction inspection and verified mutation/cache layers are complete for
the EN22 model/channel families. The next interaction milestone should build a
sensitivity/Jacobian API on the typed reversible mutation boundary, without
recreating selector arrays or text keys. Native TQBOND runtime conformance
remains limited by the absence of a non-proprietary SUBG/QUAS/QSOL data set.

The routine-by-routine native ABI audit is already complete for the current
75-wrapper surface; platform-specific binary verification remains a separate,
ongoing evidence task.
