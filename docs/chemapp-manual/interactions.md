# Interaction inspection and name resolution

This page records the high-level interaction architecture implemented after a
runtime discovery pass with the checked Win64 ChemApp library and the local
`EN22_Al-Ca-Fe-Pb-Zn-Si-O.dat` corpus. The database itself and the generated
raw descriptor dump are not repository artifacts.

## Native roles

The Programmer's Manual defines `TQLPAR` as the listing operation for all
excess Gibbs (`G`) or magnetic (`M`) interactions of one phase. It returns the
interaction count, 156-character descriptor records, and each record's actual
length. `TQGPAR` accepts the same phase/channel plus a one-based interaction
index and returns `NOEXPR × NVALA` coefficient values in default units.

The index printed at the start of every observed TQLPAR descriptor matched its
one-based position and the TQGPAR index used to retrieve its values. The Rust
wrapper now uses each returned `LGTPAR` length rather than trimming the entire
record, and reconstructs the logical TQGPAR rows from Fortran column-major
storage with the fixed expression leading dimension. Buffer capacity and
logical result shape remain separate concepts.

## Raw → parsed → optional recovery → resolved

Interaction inspection is additive:

1. `InteractionRaw` owns phase identity, model, typed channel, native parameter
   index, the exact TQLPAR output, and all authoritative live TQGPAR values.
2. `InteractionDescriptor` parses only indexed structure: powered members,
   colon-separated sublattice groups, reciprocal sides, exponent/order tokens,
   and an optional owned native type label. Unknown syntax becomes `Unparsed`.
3. An optional `InteractionDescriptorRecovery` provider may compare the native
   structure with the corresponding semantic interaction from an available
   compatible ASCII DAT model. A difference is retained as
   `InteractionDescriptorSource::DatRecovered`; the native text remains in
   `InteractionRaw`.
4. `ResolvedInteractionDescriptor` maps each usable index through ChemApp
   metadata while retaining the parsed grouping and ordering. Resolution never
   replaces or deletes the raw record.

`Phase::gibbs_interactions`, `Phase::magnetic_interactions`, and
`Phase::interactions` provide phase-local access. `Calculator::interaction_report`
collects both channels for every non-`PURE` phase. Interactions are static
loaded-model data and are deliberately not duplicated into equilibrium
snapshots or every TQMAP point.

The recovery-enabled counterparts are `Phase::interactions_with_recovery` and
`Calculator::interaction_report_with_recovery`. Recovery changes structural
identity only. TQGPAR remains the authority for the live coefficient matrix,
including parameters modified after the DAT file was loaded.
An unavailable row or provider error never aborts the interaction inventory:
the native text and values remain present and a failed recovery is reported as
an explicitly unresolved row.

## Index namespaces

Resolution is selected from `TQMODL`:

- `QKTO` and `QKTOM` descriptors use the phase-constituent namespace and are
  resolved with `TQGNPC`.
- observed `SUBQ`, `SUBL`, and `SUBLM` descriptors use one flattened namespace
  across all sublattices and are resolved with `TQNOLC` plus `TQGNLC`.
- an unverified model is not guessed; parsed rows remain explicitly unresolved.

The number of sublattices is always explicit. `PhaseInteractionReport` records
the authoritative phase count returned by `TQNOSL`, and every parsed/resolved
descriptor reports the number of sublattice groups it contains. The native
`*N` token is the interaction arity and is never used as a sublattice count.
A descriptor for `S` sublattices has `S - 1` colon separators. Resolution
rejects a descriptor whose represented count disagrees with `TQNOSL`.

Some model families have a fixed interaction structure: observed `QKTO` and
`QKTOM` descriptors have one sublattice, while `SUBQ` descriptors have two.
`SUBL` and `SUBLM` are variable-sublattice families, so their count must come
from the loaded phase rather than the model code. For sublattice population
counts `[n1, n2, ...]`, flattened one-based index ranges are
computed cumulatively. Thus `[3, 2]` maps `1..=3` to sublattice 1 and `4..=5`
to sublattice 2. This replaces the old parser's inconsistent `<`/`<=` tests
and both `+nspecies1` and `-nspecies1` offset variants. Zero and indices beyond
the cumulative total are rejected. The implementation and tests support an
arbitrary number of sublattices. EN22 includes three-sublattice ferrites and
four-sublattice Olivine rows such as `(Ca)-(Fe) : (Ca) : (Si) : (O)`.

Resolved members distinguish `PhaseConstituent { index, name }` from
`SublatticeSpecies { encoded_index, sublattice, local_index, name }`. User-facing
text therefore uses native names without mislabelling phase constituents as
sublattice species. The transformed display omits the leading native parameter
index and arity marker (`1: *2`, for example), because those remain available
as typed metadata rather than being part of the name-based thermodynamic
descriptor. It also omits diagnostic index annotations, so
`1: *2 (1)^[0]-(3)^[0] : (8) (Guts)` is rendered in the form
`(Al)^[0]-(Ca)^[0] : (O) (Guts)` after model-aware resolution.
The powered terms before the first colon form one sublattice group; every
colon begins the next sublattice group. Models such as Spinel/SUBLM use the
same group boundaries without power tokens and render, for example, as
`(A)-(B) : (C)`. Monoxide/QKTOM has only one group, so its transformed
descriptor contains no colon.

## Observed grammar matrix

The discovery corpus contained six model/channel grammar cells. All 692 native
rows were syntactically parsed and name-resolved, but DAT cross-validation
found 25 valid-looking descriptors whose wildcard order token was corrupted;
see the next section. No row was silently dropped.

| Model | Channel | Sublattices | Structural grammar | Rows | Type labels |
|---|---|---:|---|---:|---|
| `SUBQ` | Gibbs | 2 (fixed) | powered first-sublattice group plus one following sublattice group | 416 | Quasichemical (252), Guts (152), Bragg-Williams (12) |
| `SUBLM` | Gibbs | variable (`TQNOSL`) | colon-separated flattened sublattice groups | 70 | none |
| `SUBLM` | Magnetic | variable (`TQNOSL`) | colon-separated flattened sublattice groups | 35 | none |
| `SUBL` | Gibbs | variable (`TQNOSL`) | colon-separated flattened sublattice groups | 74 | none |
| `QKTO` | Gibbs | 1 (fixed) | powered phase-constituent list | 61 | none |
| `QKTOM` | Gibbs | 1 (fixed) | powered phase-constituent list | 36 | none |

Numeric powers and the literal `*` order token are retained structurally.
Native type labels are owned strings, so an unfamiliar label does not itself
make an otherwise understood descriptor fail. The manual's reciprocal syntax
is represented as distinct left and right two-member groups and is covered by
synthetic tests; no reciprocal descriptor occurred in this EN22 run.

## Known TQLPAR multi-digit-order corruption and DAT recovery

The checked ChemApp 7.14 Win64 build does not reliably print two-digit
interaction orders. In EN22 it replaced every affected numeric order with the
valid-looking token `[*]`. A parser cannot distinguish these rows from a real
wildcard using TQLPAR alone, so successful grammar parsing is not proof of
structural correctness.

Local cross-validation used `chemsage-parser` master
`785f3be35a74a99ca4b80b1c1ddcad6de7bcbb84`. TQLPAR/TQGPAR and DAT semantic
interaction counts agreed for every phase and channel. The one-based native
parameter position matched DAT declaration order for all 667 unaffected rows;
their participants, grouping, labels, and orders matched structurally. The
remaining 25 mismatches were exactly the DAT interactions whose order was 10
or greater:

| Phase(s) | Model/channel | Parameter → recovered order |
|---|---|---|
| Slag-liq#1 and Slag-liq#2 | SUBQ/G | 39→11, 40→14, 41→15, 47→15, 53→11, 59→15, 106→11, 107→13, 108→15, 115→15 |
| Zincite | QKTO/G | 10→13, 11→15, 17→15, 19→15, 21→10 |

Each native row retained its complete TQLPAR text, for example a powered member
with `^[*]`; only the structural descriptor was recovered. The local DAT
semantic views reconstructed all 25 affected rows exactly and name resolution
continued through native TQGNPC/TQGNLC metadata. TQGPAR values were never read
from or replaced by the DAT parser.

The base crate deliberately has no `chemsage-parser` dependency. That repository
is currently private and declares `publish = false`, so requiring it would make
normal public builds depend on private credentials. The public recovery trait
is the adapter boundary; a future `dat-interaction-recovery` feature should
remain default-off once the parser is publicly consumable. Recovery is only
meaningful for a compatible available ASCII DAT source, not for BIN/CST alone.

## EN22 phase coverage

`G` and `M` columns show `total/parsed/resolved`. A zero means TQLPAR reported
the documented no-data condition for that channel, which the high-level API
represents as an empty collection.

| Phase | Model | G | M |
|---|---|---:|---:|
| gas_ideal | IDMX | 0/0/0 | 0/0/0 |
| Slag-liq#1 | SUBQ | 208/208/208 | 0/0/0 |
| Slag-liq#2 | SUBQ | 208/208/208 | 0/0/0 |
| Spinel | SUBLM | 14/14/14 | 35/35/35 |
| Monoxide#1 | QKTOM | 15/15/15 | 0/0/0 |
| Monoxide#2 | QKTOM | 15/15/15 | 0/0/0 |
| a2Ca2SiO4 | QKTO | 9/9/9 | 0/0/0 |
| a-Ca2SiO4 | QKTO | 6/6/6 | 0/0/0 |
| Wollast | QKTO | 7/7/7 | 0/0/0 |
| Melilite | SUBL | 36/36/36 | 0/0/0 |
| Olivine#1 | SUBLM | 6/6/6 | 0/0/0 |
| Olivine#2 | SUBLM | 6/6/6 | 0/0/0 |
| Feldspar#1 | SUBL | 5/5/5 | 0/0/0 |
| Feldspar#2 | SUBL | 5/5/5 | 0/0/0 |
| Mullite | SUBL | 4/4/4 | 0/0/0 |
| Ca(Al,Fe)12O19 | QKTO | 1/1/1 | 0/0/0 |
| Brownmillerite | QKTO | 2/2/2 | 0/0/0 |
| Ca(Fe,Al)4O7 | QKTO | 1/1/1 | 0/0/0 |
| Mayenite | QKTO | 1/1/1 | 0/0/0 |
| C3A | SUBLM | 0/0/0 | 0/0/0 |
| M2O3(Corundum)#1 | QKTOM | 3/3/3 | 0/0/0 |
| M2O3(Corundum)#2 | QKTOM | 3/3/3 | 0/0/0 |
| CA2 | SUBLM | 0/0/0 | 0/0/0 |
| (Ca,Pb)(Al,Fe,Cr)2O4 | SUBLM | 2/2/2 | 0/0/0 |
| (Ca,Pb)(Fe,Al,Cr)2O4 | SUBLM | 2/2/2 | 0/0/0 |
| Zincite | QKTO | 22/22/22 | 0/0/0 |
| SFCA1 | SUBLM | 27/27/27 | 0/0/0 |
| Willemite | SUBLM | 0/0/0 | 0/0/0 |
| (Pb,Ca)2Fe2O5 | QKTO | 1/1/1 | 0/0/0 |
| Pb4Fe22O37 | SUBLM | 1/1/1 | 0/0/0 |
| Magnetoplumbite | SUBLM | 8/8/8 | 0/0/0 |
| Barysilite | QKTO | 1/1/1 | 0/0/0 |
| Melanotekite | QKTO | 1/1/1 | 0/0/0 |
| Pb12Fe2Si2O19 | QKTO | 0/0/0 | 0/0/0 |
| Pb9Al4(Al,Fe)4O21 | QKTO | 2/2/2 | 0/0/0 |
| Pseudowollastonite | QKTO | 0/0/0 | 0/0/0 |
| W-ferrite | SUBL | 12/12/12 | 0/0/0 |
| Larsenite | QKTO | 0/0/0 | 0/0/0 |
| Bredigite | QKTO | 0/0/0 | 0/0/0 |
| X-ferrite | SUBL | 12/12/12 | 0/0/0 |
| Garnet | SUBLM | 4/4/4 | 0/0/0 |
| Ganomalite | QKTO | 6/6/6 | 0/0/0 |
| Ca3SiO5 | QKTO | 1/1/1 | 0/0/0 |

Native syntax/name-resolution totals are Gibbs `657/657/657/0 unparsed` and
magnetic `35/35/35/0 unparsed`. Cross-source structural status is:

| Channel | Total | Native valid | Native malformed | DAT recovered locally | Unresolved after local recovery |
|---|---:|---:|---:|---:|---:|
| Gibbs | 657 | 632 | 25 | 25 | 0 |
| Magnetic | 35 | 35 | 0 | 0 | 0 |
| Total | 692 | 667 | 25 | 25 | 0 |

## Tables, examples, and cache relationship

`PhaseInteractionReport::table_string` uses the crate's shared `comfy-table`
style and shows phase, model, channel, parameter index, structural kind,
descriptor source, raw descriptor, resolved form, all coefficient rows, and
state. The combined `interactions` demo outputs every parsed and transformed
Gibbs and magnetic row. It and the focused `interactions_gibbs` and
`interactions_magnetic` examples support
`CHEMAPP_LIBRARY` plus `CHEMAPP_INTERACTION_DATAFILE` (falling back to
`CHEMAPP_DATAFILE`) and contain no workstation-specific paths.

`ParameterCache` remains a parameter perturbation/reset facility, not the
general interaction model. Its Gibbs loader now consumes the authoritative
interaction pipeline, retaining parameter index, resolved identity, and
values. Model-specific TQCDAT addressing and magnetic mutation/reset remain a
separate evidence task; listing a parameter does not prove how it may safely be
modified.

## Unknown policy

Support is claimed only for the model/channel grammars above and for the
synthetically tested reciprocal structure. Other models, labels, or syntax are
retained raw as `Unparsed` or explicitly unresolved. A valid-looking native
descriptor can also be marked DAT-recovered when deterministic cross-source
comparison finds a structural difference. The public display string is derived
from structural identity and is not the sole identity key. Native member
ordering is preserved unless a model explicitly defines symmetry.
