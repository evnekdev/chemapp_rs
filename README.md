# chemapp_rs

[![CI](https://github.com/evnekdev/chemapp_rs/actions/workflows/abi-platform-model.yml/badge.svg)](https://github.com/evnekdev/chemapp_rs/actions/workflows/abi-platform-model.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/evnekdev/chemapp_rs/blob/master/LICENSE)

`chemapp_rs` is an unofficial Rust interface for
[ChemApp](https://gtt-technologies.de/software/chemapp/), GTT Technologies'
thermochemical equilibrium programmer's library. ChemApp performs the
thermodynamic calculations; this crate dynamically loads ChemApp, adapts its
native Fortran ABI, and provides higher-level Rust workflows for calculations,
inspection, snapshots, mapping, streams, and interaction parameters.

ChemApp is proprietary software and must be obtained and licensed separately
from GTT Technologies. The crates.io package contains no ChemApp DLL/shared
library, GTT C interface source, licence, or thermodynamic database.

## Architecture in 30 seconds

```text
ChemApp DLL / SO
       |
       v
     Engine              low-level, native-oriented TQ... methods
       |
       v
   Calculator            loading and high-level workflows
       |
   +---+-----------------------+
   |              |            |
Entities       Snapshots   Interactions
live views     owned data  inspection and mutation
```

- [`Engine`](https://docs.rs/chemapp_rs/latest/chemapp_rs/struct.Engine.html) preserves ChemApp's one-based indices, options,
  state, units, and native errors with minimal Rust adaptation.
- [`Calculator`](https://docs.rs/chemapp_rs/latest/chemapp_rs/struct.Calculator.html) is the preferred starting point for most
  users. It loads a data-file and adds composition transforms, calculations,
  mapping, reports, and snapshots.
- `Calculator::engine()` is the advanced borrowed native accessor for
  intentional low-level calls; all `Engine::tq...` methods remain available.
  Raw calls that reinitialize/load another system
  invalidate Calculator metadata and must instead be performed through a new
  Calculator.
- A live entity reads the current ChemApp state. A snapshot owns copied values
  and remains valid after another calculation changes that native state.

## Requirements

You need:

- Rust 1.85 or newer;
- a compatible ChemApp native library obtained separately from GTT;
- a matching ChemApp licence where the selected library requires one;
- a thermodynamic `.dat`, `.bin`, or `.cst` data-file;
- matching process and library architectures.

A 64-bit Rust executable cannot load a 32-bit DLL/SO, and a 32-bit executable
cannot load a 64-bit library. Dynamic loading happens at runtime, so the Rust
crate and its examples compile without ChemApp being installed.

The strongest checked runtime target is the repository's historical ChemApp
7.14 Win64/x64 binary. Win32/x86 has strong ABI/source evidence, Linux/i386 is
represented by an older library with seven later exports absent, and Unix64 is
source-modelled without a checked native binary. See
[conformance](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/conformance.md) and the
[native ABI audit](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/native-abi-audit.md) before selecting a
production platform.

## Use the crate in your own project

```text
cargo new chemapp-demo
cd chemapp-demo
cargo add chemapp_rs@1
```

Equivalent `Cargo.toml` entry:

```toml
[dependencies]
chemapp_rs = "1.0"
```

Replace `src/main.rs` with:

```rust,no_run
use chemapp_rs::{Calculator, ChemAppError};

fn main() -> Result<(), ChemAppError> {
    let library = std::env::var("CHEMAPP_LIBRARY")
        .map_err(|error| ChemAppError::OtherError(error.to_string()))?;
    let datafile = std::env::var("CHEMAPP_DATAFILE")
        .map_err(|error| ChemAppError::OtherError(error.to_string()))?;
    let calculator = Calculator::from_library(&library, &datafile)?;
    println!("ChemApp version: {}", calculator.engine().tqvers()?);
    Ok(())
}
```

Then set the two paths described below and run `cargo run`. The dependency
installs only the Rust interface; ChemApp and thermodynamic data remain external.

To track current `master` instead of the stable crates.io release, use
`chemapp_rs = { git = "https://github.com/evnekdev/chemapp_rs.git" }`.

## Run the repository examples

Clone the repository and enter it before running its named examples:

```text
git clone https://github.com/evnekdev/chemapp_rs.git
cd chemapp_rs
```

The maintained examples use two environment variables:

- `CHEMAPP_LIBRARY`: path to the compatible ChemApp DLL or shared library;
- `CHEMAPP_DATAFILE`: path to a ChemApp DAT, BIN, or CST file.

PowerShell:

```powershell
$env:CHEMAPP_LIBRARY = "C:\path\to\chemapp.dll"
$env:CHEMAPP_DATAFILE = "C:\path\to\system.dat"
cargo run --example quickstart
```

POSIX shell:

```sh
CHEMAPP_LIBRARY=/path/to/libchemapp.so \
CHEMAPP_DATAFILE=/path/to/system.dat \
cargo run --example quickstart
```

The complete program is [examples/quickstart.rs](https://github.com/evnekdev/chemapp_rs/blob/master/examples/quickstart.rs). Its
core workflow is:

```rust,no_run
use chemapp_rs::{Calculator, ChemAppError};

fn main() -> Result<(), ChemAppError> {
    let required = |name: &str| {
        std::env::var(name).map_err(|error| {
            ChemAppError::OtherError(format!("could not read {name}: {error}"))
        })
    };
    let library = required("CHEMAPP_LIBRARY")?;
    let datafile = required("CHEMAPP_DATAFILE")?;
    let calculator = Calculator::from_library(&library, &datafile)?;

    println!("ChemApp version: {}", calculator.engine().tqvers()?);
    for phase in calculator.phases()? {
        println!("{}: {}", phase.name()?, phase.model()?);
    }
    Ok(())
}
```

A representative successful output shape is:

```text
ChemApp version: <native version>

System components
Index   Name
1       <component>

Phases
Index   Name                         Model
1       <phase>                      <model>
```

Names and values come from your data-file; the excerpt is deliberately not a
claimed scientific result.

## First equilibrium

[examples/equilibrium.rs](https://github.com/evnekdev/chemapp_rs/blob/master/examples/equilibrium.rs) creates a non-degenerate
unit-per-component input in
the loaded system-component basis, sets an isothermal condition, calculates,
and reports stable phases. The demonstration composition is intentionally
pedagogical rather than scientifically meaningful; replace it with the amounts
for your system. Set `CHEMAPP_TEMPERATURE` to override the default `1000.0` in
the active ChemApp temperature unit. Set `CHEMAPP_PRESSURE` to override the
explicit default `1.0` in the active pressure unit.

```powershell
cargo run --example equilibrium
```

The example uses `Calculator::calculate_isothermal_at_pressure`, so pressure is
visible and explicit. `calculate_isothermal` is the convenience counterpart:
it resets conditions with `TQREMC(-2)` and therefore uses ChemApp's documented
1-bar default. Both convert mole amounts from the active user basis, set
temperature and incoming system-component amounts, call no-target `TQCE`, and
leave the resulting equilibrium as the live ChemApp state.

## Live entities and owned snapshots

Components, phases, constituents, sublattice species, and model-aware TQBOND
pair/quadruplet entities are live views. Their getters query the current native
state and return `Result`.

Use a live entity's `snapshot()` method—or `calculator.snapshot()` for a deep
system copy—before changing conditions when an earlier result must remain
available. Snapshot types implement `Debug` and `Clone`.
`SnapshotOptions::stable_only()` applies the exact project criterion
`AC > 0.9999`; it is not an approximate hidden heuristic. Run
[examples/snapshots.rs](https://github.com/evnekdev/chemapp_rs/blob/master/examples/snapshots.rs) for the live/owned contrast.

Temperature and pressure mapping APIs return a snapshot for every successful
native mapping state, including the terminal state:

- `mapping_temperature` / `mapping_temperature_with_options`;
- `mapping_pressure` / `mapping_pressure_with_options`.

See [entities and snapshots](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/entities-and-snapshots.md) for
state, filtering, table, TQBOND, and mapping details.

## Streams

`entities::stream::Stream` owns one high-level Rust stream name. ChemApp names
streams globally within an engine, so a calculator permits one live owner per
name. Use `Stream::remove()` when cleanup errors must be observable. Normal
`Drop` cleanup is best-effort because destructors cannot return a native error.
The dataset-specific [entities demo](https://github.com/evnekdev/chemapp_rs/blob/master/examples/entitiesdemo.rs) exercises stream
creation, snapshots, tables, and mapping.

## Inspect interactions

[examples/interactions.rs](https://github.com/evnekdev/chemapp_rs/blob/master/examples/interactions.rs) prints Gibbs and magnetic
interactions for every solution phase. Set `CHEMAPP_PHASE` to one exact phase
name to limit the report. Each row retains:

- the raw TQLPAR descriptor;
- the parsed and name-resolved structural descriptor;
- descriptor provenance/cross-check status;
- the complete live TQGPAR matrix;
- typed mutation support or an explicit read-only reason.

A phase may legitimately have no magnetic interactions. Focused
`interactions_gibbs` and `interactions_magnetic` examples are also provided.
No interaction example requires `chemsage-parser` or an ASCII-DAT recovery
provider.

Read [interaction inspection](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/interactions.md) for model
grammars, sublattices, provenance, and the known two-digit-order TQLPAR defect.

## Advanced: interaction parameter mutation

[examples/parameter_mutation.rs](https://github.com/evnekdev/chemapp_rs/blob/master/examples/parameter_mutation.rs) locates a
runtime-verified `InteractionParameterAddress`, reads it with
`Calculator::interaction_parameter`, performs a tiny temporary write, verifies
readback, and restores the exact baseline.

This is an advanced API:

- TQCDAT changes the loaded model in memory;
- prior equilibrium results become stale;
- it does not modify the source DAT unless a separate write routine is called;
- SUBQ columns 7–18 and other unverified model-specific terms remain read-only;
- phase-local addresses may be aliases when ChemApp loaded multiple copies of
  one phase, so cached mutable cells are addressable views, not necessarily
  independent thermodynamic parameters.

The example never calls TQWASC. Read
[parameter mutation](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/parameter-mutation.md) before using
this interface in sensitivity or fitting work.

## Capability overview

The 1.0 API includes:

- runtime ChemApp library loading and DAT/BIN/CST input;
- 75 low-level `Engine::tq...` wrappers;
- high-level `Calculator` calculation and composition-basis workflows;
- live system/component/phase/constituent/species/TQBOND entities;
- engine-independent snapshots and deterministic tables;
- stable-only snapshots and temperature/pressure mapping;
- uniquely owned high-level streams;
- Gibbs and magnetic interaction inspection and index-to-name resolution;
- complete TQGPAR matrices;
- typed TQCDAT mutation for runtime-verified model/channel families;
- baseline-relative parameter caching and verified reset support.

Parameter mutation and raw `Engine` calls are advanced. Low-level methods
preserve ChemApp semantics and may require knowledge of the official
Programmer's Manual. Native interaction addresses and parameter-cache entries
are local to the loaded system; do not persist and reuse them with another
data-file/configuration.

## Examples

| Example | Level | Purpose |
|---|---|---|
| `quickstart` | beginner | Load and inspect an arbitrary system |
| `equilibrium` | beginner | Perform a first isothermal equilibrium |
| `snapshots` | intermediate | Preserve owned results across native state changes |
| `interactions` | intermediate | Inspect raw, resolved, Gibbs, and magnetic interactions |
| `interactions_gibbs` | intermediate | Print only Gibbs interactions |
| `interactions_magnetic` | intermediate | Print only magnetic interactions |
| `parameter_mutation` | advanced | Reversibly mutate one verified interaction cell |
| `entitiesdemo` | advanced/dataset-specific | Exercise entities, streams, tables, snapshots, and mapping |
| `maindemo` | low-level conformance | Broad Rust translation of GTT's C demonstration sequence |

Examples require ChemApp to run, but not to compile. `entitiesdemo` and
`maindemo` require a C-O-Si dataset with the names used by the original demo;
the beginner examples avoid dataset-specific phase names.

## Troubleshooting

### Native library not found

Use an absolute `CHEMAPP_LIBRARY` path and confirm the process can read it.
`Engine::new` returns the dynamic-loader diagnostic; it does not search for or
install ChemApp.

### Wrong architecture or unsupported binary

Match Rust target and library bitness. A symbol-not-found error can also mean
the ChemApp binary predates a routine; the checked Linux/i386 library lacks
seven later data-manipulation exports.

### Licence or dongle error

Confirm the selected ChemApp edition and local GTT licensing setup. Do not post
licence-holder names, user IDs, dongle IDs, or tokens in public bug reports.

### Data-file cannot be opened

Confirm `CHEMAPP_DATAFILE`, access permissions, and extension. The high-level
loader accepts `.dat`, `.bin`, and `.cst` case-insensitively and uses ChemApp's
format-specific open/read/close calls.

### An interaction channel is empty

Not every phase/model has excess Gibbs or magnetic parameters. Empty magnetic
output is valid. Use `TQMODL`/the phase model and inspect the other channel.

### CI or docs.rs has no ChemApp licence

Builds and pure tests do not load ChemApp. Dynamic loading occurs only when an
application constructs an `Engine`, so docs.rs and normal source checks need no
commercial binary, licence, or data-file.

## Detailed documentation

- [Entities, snapshots, tables, and mapping](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/entities-and-snapshots.md)
  explains live state, ownership, filtering, TQBOND, and continuation.
- [Interactions](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/interactions.md) documents parsing,
  sublattice grouping, name resolution, and provenance.
- [Parameter mutation](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/parameter-mutation.md) records exact
  TQGPAR/TQCDAT selectors, verified families, cache semantics, and limitations.
- [Best practices](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/best-practices.md) translates official
  ChemApp guidance into Rust rules.
- [Conformance](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/conformance.md) summarizes current runtime
  and platform evidence.
- [Native ABI audit](https://github.com/evnekdev/chemapp_rs/blob/master/docs/chemapp-manual/native-abi-audit.md) contains the full
  routine-by-routine direct-Fortran audit.

## API stability

Version 1.0 is the first stable release and establishes the current public API.
Compatible additions and fixes follow SemVer; incompatible public API changes
require a new major version. Low-level wrappers aim to
preserve ChemApp semantics across compatible releases. Advanced mutation
support remains deliberately conservative: additional models or terms are
enabled only after direct runtime evidence establishes their selectors.

## Contributing

Issues and pull requests are welcome. Include the Rust target, ChemApp version,
library architecture, failing routine, and a minimal reproducible call sequence
when possible. Do not attach proprietary ChemApp binaries, commercial
databases, or licence-specific data unless you have explicit redistribution
permission.

## Licence and trademarks

Project-authored Rust code and documentation are licensed under the
[MIT License](https://github.com/evnekdev/chemapp_rs/blob/master/LICENSE). ChemApp is proprietary software of GTT Technologies and
is not covered by this project's MIT licence. `chemapp_rs` is unofficial and is
not endorsed by GTT Technologies.
