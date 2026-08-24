# Interaction parameter addressing and reversible mutation

This page records the verified relationship between TQLPAR interaction
identity, the complete TQGPAR parameter matrix, and TQCDAT mutation selectors.
The primary semantic authority is Programmer's Manual §§6.2–6.4. Runtime
evidence below is limited to ChemApp 7.14 in the checked Win64
`windows/ca_vc_e_x64.dll` and the local, uncommitted EN22 ASCII DAT corpus.

## Typed address rules

All Rust-facing identities are one-based, matching ChemApp. Vector offsets are
zero-based only inside matrix adapters.

| Parameter | Typed Rust address | Exact TQCDAT selectors `I1..I5` |
|---|---|---|
| Excess Gibbs term | `Gibbs { phase_index, interaction_index, expression_index, term_index }` | `13, interaction, expression, term, phase` |
| Excess Curie/Neel temperature | `Magnetic { ..., role: CurieNeelTemperature }` | `10, interaction, expression, 1, phase` |
| Excess magnetic moment | `Magnetic { ..., role: MagneticMoment }` | `10, interaction, expression, 2, phase` |

`InteractionParameterAddress` structurally separates Gibbs terms from the two
magnetic roles. Zero indices are rejected. `Calculator::interaction_parameter`
reads the containing live TQGPAR matrix and performs checked row/column lookup.
`Calculator::set_interaction_parameter` first proves the cell exists and that
its model/channel family is in the verified set, then lowers the typed address
to TQCDAT. Raw `Engine::tqcdat` remains the unrestricted low-level escape hatch.

TQGPAR values remain authoritative even when an optional DAT cross-check
recovers a damaged TQLPAR descriptor. Pretty or resolved descriptor text is
never a mutation key.

## Matrix and buffer semantics

TQGPAR returns a logical `NOEXPR × NVALA` matrix from Fortran column-major
storage. The native `VALA` leading dimension is TQSIZE `NI`, not returned
`NOEXPR`; the checked runtime reports `NI=20`. Rust now queries that value,
allocates the flat buffer using it, validates dimensions, and reconstructs all
logical rows. A synthetic `NI=5`, `NOEXPR=2`, `NVALA=3` test prevents using
`NOEXPR` as the stride.

The checked source/runtime supports a version-scoped second extent of 28. The
manual's older declaration describes 18 columns, and EN22 returned at most 18.
The value 28 is therefore retained as checked-build capacity, not a universal
ChemApp limit.

TQLPAR requires enough 156-character records for the selected channel. Manual
§2.11 defines TQSIZE `ND` as the maximum excess Gibbs coefficient/list extent
and `NE` as the maximum excess magnetic extent; §6.2 has inconsistent wording
that calls the magnetic extent `ND`. The channel meaning and the checked bridge
support `ND` for `G` and `NE` for `M`. Rust now queries TQSIZE and allocates
accordingly. The checked runtime reports `ND=2000`, `NE=500`; TQUSED reports
`ND=209`, `NE=36`. The historical fixed 1,999-record buffer is no longer used.

## Win64 EN22 verification matrix

The inventory contained 692 interactions and 9,034 matrix cells:

| Model | Channel | `NOEXPR × NVALA` | Rows | Read | Mutate | Evidence |
|---|---|---:|---:|---|---|---|
| SUBQ | Gibbs | `1 × 18` | 416 | yes | terms 1–6 only | exhaustive Win64 round-trip; terms 7 and 18 explicitly rejected |
| SUBL | Gibbs | `1 × 6` | 74 | yes | yes | exhaustive Win64 round-trip |
| SUBLM | Gibbs | `1 × 6` | 67 | yes | yes | exhaustive Win64 round-trip |
| SUBLM | Gibbs | `2 × 6` | 1 | yes | yes | exhaustive first/last expression and all terms |
| SUBLM | Gibbs | `3 × 6` | 2 | yes | yes | exhaustive first/middle/last expression and all terms |
| SUBLM | Magnetic | `1 × 2` | 35 | yes | both roles | exhaustive Win64 round-trip |
| QKTO | Gibbs | `1 × 6` | 61 | yes | yes | exhaustive Win64 round-trip |
| QKTOM | Gibbs | `1 × 6` | 36 | yes | yes | exhaustive Win64 round-trip |

All 4,042 verified cells were independently changed, the entire containing
matrix reread, only the target cell observed to differ, the exact original f64
restored, and the entire matrix compared again. This exhaustively covers first,
middle, and last ordinary Gibbs terms; both magnetic roles; and every row of
the multi-expression matrices. Six additional representative cycles confirmed
the previous/next interaction remained unchanged, ruling out an observed
INDEXX off-by-one error.

The remaining 4,992 cells are SUBQ columns 7–18. The documented generic Gibbs
selector was explicitly attempted for both term 7 and term 18; ChemApp returned
error 1024 and left the full matrix unchanged. These cells remain inspectable
and cached as read-only. They must not be guessed from the 18-column shape.
No pressure-dependent interaction was independently identified, so pressure
term mutation remains unverified. SUBG special power/term mutation likewise
remains documented but runtime-unverified because EN22 has no SUBG corpus.

The exhaustive audit also included recovered descriptor rows
Slag-liq#1/SUBQ parameter 39 and Zincite/QKTO parameter 10. Dedicated repeat
cycles succeeded. This confirms that phase/channel/native interaction index,
not recovered text, controls TQGPAR/TQCDAT addressing.

## Duplicate phase copies and equilibrium state

The manual states that TQCDAT changes every copy of a duplicated phase.
Slag-liq#1 and Slag-liq#2 interaction 1 were read before and after a mutation
through copy #1. The selected value changed identically in copy #2; restoring
through copy #1 restored both complete matrices exactly. This is documented
native propagation, not accidental cache cross-talk.

The high-level API therefore treats duplicate phase copies as separately
addressable phase-local views over native parameters whose writes can propagate
to every copy. The inspected native metadata does not expose a durable copy-
family identifier, and a display-name suffix such as `#1` or `#2` is not strong
enough evidence to manufacture one. `ParameterCache` consequently neither
claims that copies own independent parameters nor invents an alias-group API;
callers must assume the documented TQCDAT propagation rule.

One representative parameter from each of the six observed model/channel
families was also mutated across an equilibrium calculation, restored, and
recalculated successfully. A separate active-phase Gibbs probe recorded system
Gibbs energy, observed a finite response, restored the exact coefficient, and
reproduced the baseline result within relative tolerance. TQGPAR readback—not
physical response magnitude—is the addressing proof.

TQCDAT changes the loaded in-memory model. It does not recalculate or clear
conditions, and prior equilibrium results become stale. Callers explicitly
control the next calculation. The probe used a disposable Engine, never called
TQWASC, restored every accepted mutation, and verified the source DAT digest
was unchanged.

## ParameterCache contract

`ParameterCache` is model-neutral. It retains all 9,034 TQGPAR cells, including
read-only ones, and indexes mutable cells by
`phase_index/channel/interaction_index/expression_index/column_or_role`.
Native and resolved text are display/provenance fields only.

The cache captured 4,042 verified mutable cells. Absolute writes use the
supplied value. Delta writes always mean `captured_baseline + delta`; repeated
identical delta writes do not accumulate from live state. Parameter-level,
interaction-level, interaction-surface, and full-cache resets write the stored
typed addresses and verify TQGPAR readback. Runtime checks exercised magnetic
moment and multi-expression Gibbs delta/reset paths.

The former MQM-specific interaction cache types and six-term/first-expression
assumptions were removed. This is an API change in the experimental cache
surface.

## Remaining limitations

- SUBG TQGPAR/TQCDAT special power semantics lack a suitable runtime corpus.
- SUBQ columns 7–18 are read-only until their model-specific selectors are
  established; the generic selector is known not to work in the checked build.
- Pressure-dependent Gibbs mutation has not been separately identified or
  verified.
- The 28-column allocation is checked-build/version evidence, not a portable
  maximum.
- Linux/i386 lacks these later data-manipulation exports and Unix64 has no
  checked native binary.

The next sensitivity/Jacobian layer should consume the typed mutation and
verified-reset APIs; it must not recreate raw selector arrays or text keys.
