# Entities, snapshots, tables, and mapping

This page defines the high-level state model built over the direct ChemApp
Fortran bindings. The semantic source is the official Programmer's Manual;
the low-level ABI remains documented separately in
[abi-and-c-interface.md](abi-and-c-interface.md).

## Live entities and immutable snapshots

`System`, `SystemComponent`, `Phase`, `Constituent`, `Species`, `Bond`, and
`Stream` are live views of the current mutable ChemApp state. Their native
property accessors return `Result`; a native error is never converted to
`NaN`, `false`, an empty iterator, or a placeholder name on the authoritative
path.

Snapshots own all data needed after the engine advances. Snapshot creation is
fallible and never fabricates a partial state. `CalculatorSnapshot` stores the
active temperature, pressure, volume, energy, and amount units once, plus the
whole-system state, all system components, and the retained phase hierarchy.
Each retained phase owns its component `XP`/`AP` relations, constituents,
sublattice species, and applicable TQBOND results. `StreamSnapshot` is an
independent owned stream record.

`SnapshotOptions::all()` is the default. `SnapshotOptions::stable_only()`
filters phases before querying their deep descendants. The sole project
stability rule is the strict comparison:

```text
stable phase iff AC > 0.9999
```

The amount `A` is not used for this filter. Each snapshot records whether the
filter was requested so omitted phases are not ambiguous.

## TQBOND is model-dependent

TQBOND is a model-dependent result interface, not a universal quadruplet
interface. The semantic rules below follow the official
[ChemApp Programmer's Manual, TQBOND §5.13](https://gtt-technologies.de/ca-doc/index.html):

| TQMODL base code | High-level result | Identity supplied to TQBOND |
| --- | --- | --- |
| `SUBG` | quadruplet fraction | four sublattice-constituent indices |
| `QUAS` | pair fraction | two phase-constituent indices |
| `QSOL` | pair fraction | two phase-constituent indices |
| every other model | not applicable | no `Bond` entities |

The public name `Bond` is retained, but it means one high-level TQBOND pair or
quadruplet result. `BondKind` makes that distinction explicit; pairs never
contain dummy third or fourth members. `BondSnapshotKind` preserves the same
distinction with owned member names and indices.

For `QUAS` and `QSOL`, the manual defines `INDEXA` and `INDEXB` as ordinary
one-based phase-constituent indices and permits either order. The iterator
canonicalizes them as `(min, max)`, includes like-member pairs such as `A-A`
once (the manual does not exclude repeated indices), and never also emits
`B-A`. The high-level adapter passes zero in unused raw `INDEXC`/`INDEXD`
slots. The checked manual section does not prescribe a distinct mandatory
dummy value for those unused arguments; the zeroes are neutral adapter
placeholders and are not exposed as member identity.

For `SUBG`, high-level member identity remains
`SpeciesRef { sublattice, local_index }`. Two canonical members belong to
sublattice 1 and two to sublattice 2. Within each sublattice, member order is
canonicalized and combinations with replacement are enumerated once. No
additional cross-sublattice symmetry is assumed. Only at the native boundary
is a second-sublattice local index converted to:

```text
number of constituents in sublattice 1 + local index in sublattice 2
```

`Phase::bonds()` queries TQMODL before constructing the iterator. Unsupported
models produce an empty iterator without speculative TQBOND calls. A SUBG
phase must expose exactly the two sublattices required by the documented
quadruplet encoding; an inconsistent model structure is reported as an error.

## Iteration and validation

Component, phase, and constituent iterator constructors query their native
counts fallibly. Species enumeration flattens applicable `SUB*` sublattices
while retaining each local sublattice identity. Bond enumeration is a
model-dispatched combinatorial iterator, not a scalar counter.

`Constituent::is_valid`, `Species::is_valid`, and `Bond::is_valid` validate
the complete one-based identity and propagate count/model query failures.
Pair and quadruplet validation apply different rules; there is no generic
four-positive-index shortcut.

## Tables

Tables use `comfy-table` 8 without terminal detection or default ANSI styling.
Numeric values use one deterministic scientific formatter. Live tables are
fallible; snapshot `Display` is infallible and performs no native calls.

One row/schema implementation is shared by each live/snapshot entity pair.
The calculator report contains separate system, component, phase,
phase-component, constituent, species, and TQBOND tables. TQBOND rows use the
common columns `Phase`, `Model`, `Kind`, `Members`, and `X`, for example a
pair as `A [1] - B [2]` and a quadruplet as
`A [S1:1], B [S1:2] | C [S2:1], D [S2:2]`. Pair rows never have fake empty
member columns. Streams use their own shared live/snapshot table.

## Mapping state machine

The high-level temperature and pressure mapping methods implement ChemApp's
stateful continuation protocol exactly:

1. call `TQMAP` (or `TQMAPL` when `list == true`) with `TF`/`PF`;
2. snapshot that current native result;
3. while `ICONT > 0`, call with `TN`/`PN` and immediately snapshot again;
4. retain the final successful call whose continuation is non-positive.

Snapshots remain in native call order; they are neither sorted nor
deduplicated. The phase and constituent indices are forwarded independently.
Options-bearing mapping methods apply full or stable-only filtering during
each snapshot, not after the map.

## Runtime evidence and limits

On the checked Win64 DLL, the project-relative `entitiesdemo` used
`data/cosi.dat` and the manual's stream-style quartz/CO2 input. It successfully
captured full and stable-only snapshots, verified live/snapshot table equality
for an unchanged state, and completed unlisted full, unlisted stable-only, and
listed stable-only temperature maps. The equilibrium state contained eight
phases in the full snapshot and two under the strict stable-only rule. The
tested interval returned two native states in each mapping mode, including the
final continuation result.

The checked `cosi.dat` system contains no SUBG, QUAS, or QSOL phase, so the
TQBOND model dispatch, offset, canonicalization, and duplicate-prevention
coverage is pure structural evidence rather than a native model-data run.
Unix64 remains without a checked native binary.
