# ChemApp subroutine index for `chemapp_rs`

Primary legacy reference: https://gtt-technologies.de/ca-doc/index.html

Secondary current reference for newer releases: https://python.gtt-technologies.de/doc/chemapp/

This page is an engineering map, not a substitute for the official routine descriptions. Before changing any wrapper, read the complete official section for that routine.

## Layering rule

`src/native.rs` should remain the low-level, semantics-preserving wrapper over native ChemApp operations. Higher-level behavior belongs in `Calculator`, entities, iterators, snapshots, or future typed API modules.

## 2. Initialization, licensing, I/O, files, and units

| Native routine | Purpose | `chemapp_rs` status |
| --- | --- | --- |
| `TQINI` | initialize/reset ChemApp interface | wrapped |
| `TQCPRT` | copyright message | wrapped |
| `TQVERS` | get run-time ChemApp version | wrapped |
| `TQLITE` | detect ChemApp light | wrapped |
| `TQGTID` | get user ID | wrapped |
| `TQGTNM` | get license-holder name | wrapped |
| `TQGTPI` | get program ID | wrapped |
| `TQGTHI` | get dongle/HASP information | wrapped |
| `TQGTED` | get expiry information | wrapped |
| `TQCONF` | configure ChemApp options | wrapped |
| `TQSIZE` | get compiled internal array capacities | wrapped |
| `TQUSED` | get dimensions used by loaded system | wrapped |
| `TQGIO` | query I/O unit/language configuration | wrapped |
| `TQCIO` | change I/O unit/language configuration | wrapped |
| `TQRFIL` | read ASCII thermochemical data | wrapped |
| `TQRBIN` | read binary thermochemical data | wrapped; legacy format deprecated in manual |
| `TQRCST` | read transparent thermochemical data | wrapped |
| `TQOPEN` | open file by FORTRAN unit, primarily for output in newer legacy versions | wrapped |
| `TQWSTR` | write text through ChemApp LIST/ERROR infrastructure | wrapped |
| `TQOPNA` | open existing ASCII data-file | wrapped |
| `TQOPNB` | open existing binary data-file | wrapped |
| `TQOPNT` | open transparent data-file | wrapped |
| `TQCLOS` | close ChemApp-associated file/unit | wrapped |
| `TQGTRH` | retrieve transparent-file header | wrapped |
| `TQGSU` | get active system unit | wrapped |
| `TQCSU` | change active system unit | wrapped |

## 3. Chemical-system identification and status

| Native routine | Purpose | `chemapp_rs` status |
| --- | --- | --- |
| `TQINSC` | system component name -> index | wrapped |
| `TQGNSC` | component index -> name | wrapped |
| `TQCNSC` | rename/change component name | wrapped |
| `TQNOSC` | number of system components | wrapped |
| `TQSTSC` | component stoichiometry and molecular mass | wrapped |
| `TQCSC` | change system-component basis | wrapped |
| `TQINP` | phase name -> index | wrapped |
| `TQGNP` | phase index -> name | wrapped |
| `TQMODL` | phase model identifier | wrapped |
| `TQNOP` | number of phases | wrapped |
| `TQINPC` | phase constituent name -> index | wrapped |
| `TQGNPC` | constituent index -> name | wrapped |
| `TQPCIS` | whether constituent is permitted as incoming species | wrapped |
| `TQNOPC` | number of phase constituents | wrapped |
| `TQSTPC` | constituent stoichiometry/molecular mass | wrapped |
| `TQCHAR` | charge information | wrapped |
| `TQINLC` | sublattice constituent name -> index | wrapped |
| `TQGNLC` | sublattice constituent index -> name | wrapped |
| `TQNOSL` | number of sublattices | wrapped |
| `TQNOLC` | number of constituents in a sublattice | wrapped |
| `TQGSP` | get phase status | wrapped |
| `TQCSP` | change phase status | wrapped |
| `TQGSPC` | get phase-constituent status | wrapped |
| `TQCSPC` | change phase-constituent status | wrapped |

The `entities` and `iterator` modules form higher-level Rust views over much of this group.

## 4. Defining equilibrium calculations

| Native routine | Purpose | `chemapp_rs` status |
| --- | --- | --- |
| `TQSETC` | set global equilibrium condition / incoming amount / target | wrapped |
| `TQREMC` | remove/reset conditions and targets | wrapped |
| `TQSTTP` | define stream and set T/P | wrapped |
| `TQSTCA` | set stream constituent amount | wrapped |
| `TQSTEC` | set condition when input is stream-based | wrapped |
| `TQSTRM` | remove stream | wrapped |

Important: global conditions and streams are distinct input modes. `TQSETC` and `TQSTCA`/`TQSTEC` must not be mixed as though they were interchangeable.

## 5. Calculation and result retrieval

| Native routine | Purpose | `chemapp_rs` status |
| --- | --- | --- |
| `TQCE` | calculate equilibrium | wrapped |
| `TQCEL` | calculate equilibrium and produce result table | wrapped |
| `TQCEN` | continue/recalculate using previous information | wrapped |
| `TQCENL` | listed/table-producing counterpart | wrapped |
| `TQMAP` | one-dimensional phase mapping | wrapped |
| `TQMAPL` | mapping with list/table output | wrapped |
| `TQCLIM` | set target/mapping variable limits | wrapped |
| `TQSHOW` | show current input conditions/settings | wrapped |
| `TQGETR` | retrieve result from current calculated state | wrapped |
| `TQGDPC` | get thermodynamic property of a phase constituent | wrapped |
| `TQSTXP` | retrieve stream thermodynamic property | wrapped |
| `TQGTLC` | sublattice fraction/result | wrapped |
| `TQBOND` | bond/pair result information for applicable models | wrapped |
| `TQERR` | retrieve current ChemApp error message | wrapped |

`TQGETR` refers to the last relevant calculation/mapping state. High-level Rust code that needs historical states must snapshot results before the next native calculation changes them.

## 6. Thermodynamic data manipulation

| Native routine | Purpose | `chemapp_rs` status |
| --- | --- | --- |
| `TQGDAT` | read selected thermodynamic data | wrapped |
| `TQLPAR` | list interaction/model parameter descriptions | wrapped |
| `TQGPAR` | get parameter values | wrapped |
| `TQCDAT` | change thermodynamic data | wrapped |
| `TQWASC` | write ASCII thermochemical data-file | wrapped |

The current `cache` and `parse` modules build on this group. Their support is incomplete for some parameter/model classes and should be treated as advanced/experimental until audited.

## Current manual-to-code audit target

For every routine above, the eventual audit should record:

- manual section number;
- first ChemApp version documented to support it;
- C and FORTRAN signatures;
- actual 32/64-bit Windows and Unix exported symbol/signature used by our supported libraries;
- input/output parameters;
- string-length/hidden-length requirements;
- allowed option values;
- native one-based index semantics;
- state prerequisites and state mutation;
- units;
- documented errors;
- whether the current Rust signature is exact;
- conformance tests/examples.

## Newer ChemApp API surface

Current official ChemApp for Python documentation for newer ChemApp 8.x releases exposes additional native concepts/routines not represented in the legacy manual's chapter-6 list or in the current `defs.rs` map, including function-expression and function-sum operations such as `tqcfct`, `tqgfct`, `tqefct`, `tqcsum`, and related calls.

These are **version-gap candidates**, not automatic work items. Before adding them:

1. confirm they exist in the actual ChemApp libraries we intend to support;
2. obtain the native signature/ABI from authoritative current documentation or headers;
3. decide the minimum ChemApp version for the wrapper;
4. implement feature/version detection rather than assuming every supported library exports them.