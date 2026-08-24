# ChemApp concepts and state model

Primary reference: https://gtt-technologies.de/ca-doc/index.html

This page summarizes ChemApp concepts that directly constrain `chemapp_rs` design. It is intentionally a paraphrase, not a replacement for the official manual.

## ChemApp is a stateful calculation engine

A loaded ChemApp library instance owns mutable internal state: initialization settings, loaded thermodynamic system, active units, phase/constituent statuses, equilibrium conditions, streams, configuration options, the last calculated equilibrium, and mapping state.

Therefore an `Engine` must be treated conceptually as a state machine even where Rust FFI methods currently take `&self`. Calls are not independent pure functions and call order matters.

Do not assume that simultaneous calls into the same loaded ChemApp instance are re-entrant or independent. Parallel calculations should use independently loaded ChemApp instances, consistent with the project's multi-library strategy.

## Initialization and data loading

`TQINI` initializes ChemApp and must precede normal use. It also restores the complete set of default values and units when called again.

Thermochemical data are then loaded from a ChemApp data-file. The legacy manual describes three formats:

- ASCII: open with `TQOPNA`, read with `TQRFIL`.
- Transparent: open with `TQOPNT`, read with `TQRCST`.
- Binary: open with `TQOPNB`, read with `TQRBIN`; the legacy manual already marks this format deprecated.

An opened ChemApp data-file should be closed with `TQCLOS` after reading.

The manual recommends obtaining the data-file FORTRAN unit through `TQGIO("FILE")` instead of assuming a particular unit number.

Once a thermochemical system has been loaded, phases or phase constituents cannot simply be appended to it. For ASCII data, selected thermodynamic parameters may be changed through the data-manipulation routines. System components and statuses can also be changed through the appropriate system-data routines.

## System structure

The main hierarchy is:

- thermochemical system
- system components
- phases
- phase constituents
- for applicable models: sublattices, sublattice constituents, species, bonds, and related model-specific entities

A fixed-composition phase effectively contains one constituent. Mixture phases contain multiple constituents governed by a solution model.

Names used for ordinary components, phases and phase constituents are historically limited to 24 characters in the native interface. Some routines dealing with IDs, headers, messages, or stream identifiers use longer strings and must be handled according to their own documented signatures.

## Native indexing

The native ChemApp interface is fundamentally one-based for component, phase, constituent, and sublattice indices.

Identity-sensitive code must obtain indices at run time from names using routines such as:

- `TQINSC` for a system component
- `TQINP` for a phase
- `TQINPC` for a phase constituent
- `TQINLC` for a sublattice constituent

The corresponding `TQGN...` routines translate indices back to names.

`Engine` should preserve ChemApp's native index semantics. Higher-level Rust types may hide raw indices or expose safer typed identity, but must not make the low-level API ambiguous about whether an index is native/one-based or Rust/zero-based.

## Phase and constituent statuses

ChemApp distinguishes statuses such as `ENTERED`, `DORMANT`, and `ELIMINATED`.

Broadly:

- entered entities participate normally;
- dormant constituents can participate in activity calculations while being excluded from mass balances, depending on the model;
- eliminated entities are excluded from the equilibrium calculation.

There are model-specific restrictions on which constituent statuses may be changed. Do not generalize status changes beyond what the native routine allows.

## Two mutually exclusive ways to define initial conditions

ChemApp supports two principal input models, and the manual explicitly says they must not be used simultaneously.

### Global conditions

Use `TQSETC` to define system conditions and incoming amounts directly.

Typical simple equilibrium input consists of temperature, pressure or volume, and incoming composition. Incoming material can be entered using system components or permitted phase constituents.

`TQSETC` returns a condition number for conditions that may later need to be removed.

### Streams

Use `TQSTTP`, `TQSTCA`, and `TQSTEC` when the input is represented by streams. This method is especially important when reaction extensive-property balances are required or when reactor/process inputs are naturally represented as streams.

`TQSETC` and the stream-condition path (`TQSTCA`/`TQSTEC`) are not interchangeable.

## Important `TQSETC` semantics

The meaning of `INDEXP` and `INDEX` depends on the variable being set. For component/phase/constituent conditions the native convention is:

| `INDEXP` | `INDEX` | Entity |
| --- | --- | --- |
| `> 0` | `> 0` | constituent of a phase |
| `> 0` | `<= 0` | phase |
| `<= 0` | `> 0` | system component |
| `0` | `0` | entire system |

Pressure and temperature have documented defaults in the legacy manual (1 bar and 1000 K respectively), but high-level code should not silently depend on defaults when explicit conditions improve reproducibility.

For a phase-amount (`A`) target:

- non-negative `VAL` defines a formation phase target;
- negative `VAL` defines a precipitation phase target for a mixture phase.

Thus the sign is semantic; a particular negative magnitude should not be interpreted by high-level code as a physical phase amount.

## Removing/resetting conditions

`TQREMC` accepts either a condition number returned by `TQSETC` or documented special values:

- `0`: remove incoming-amount conditions but leave other conditions/targets;
- `-1`: remove all conditions, targets, and configuration options and reset system units to defaults;
- `-2`: same broad reset, but preserve current system units.

`Calculator::reset()` currently uses `TQREMC(-2)`, so its intended semantic is a calculation-state reset that preserves unit configuration.

## Target calculations

A target calculation has two distinct pieces:

1. the **target condition**, set using `TQSETC` or `TQSTEC`;
2. the **target variable**, supplied when calling `TQCE` or `TQCEL`.

Examples include:

- extensive-property targets such as enthalpy or volume;
- formation phase targets: determine when a specified phase becomes stable;
- precipitation phase targets: determine when a second phase becomes stable relative to a specified mixture phase.

ChemApp light has restrictions on phase targets and mapping; high-level APIs must detect capability through `TQLITE` or propagate the native error rather than assuming the full version.

## One-dimensional phase mapping

`TQMAP`/`TQMAPL` search an interval for all phase transitions along one search variable. The legacy manual identifies pressure, temperature, and incoming amount as mapping variables.

Mapping is iterative/stateful: a mapping operation returns one result at a time and an indicator of whether additional calls are necessary. Code must preserve every result before advancing ChemApp to the next one.

This is why `CalculatorSnapshot` is architecturally important: live entity objects refer to current engine state, while snapshots materialize a result before another native call changes that state.

## Result lifetime

`TQGETR` obtains results from the most recent successful calculation/mapping operation. The value exposed by a live `Phase`, `SystemComponent`, or similar entity is therefore a query against current ChemApp state, not immutable historical data.

Any API that needs to retain multiple calculated states must copy them into Rust-owned data.

## Units are mutable state

ChemApp has active units for pressure, volume, temperature, energy, and amount. `TQGSU` queries them; `TQCSU` changes them.

The manual explicitly warns against assuming units in application output because another part of a program may have changed them. High-level APIs should either:

- query and expose the active ChemApp units, or
- provide an explicit unit policy and enforce it.

Changing amount units can also change the interpretation of quantities such as molecular masses returned in terms of the current amount unit per mole.

## Data manipulation

The data-manipulation routines (`TQGDAT`, `TQLPAR`, `TQGPAR`, `TQCDAT`, `TQWASC`, plus version-dependent newer routines) operate on thermodynamic model data rather than ordinary equilibrium state.

The legacy manual states that modification of thermodynamic data is available when the loaded data-file is ASCII. `ParameterCache` therefore belongs to a distinct, more advanced layer than ordinary equilibrium calculation and should enforce format/version/model limitations explicitly.