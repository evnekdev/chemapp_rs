//! Dataset-specific entity, stream, snapshot, and mapping demonstration.
//!
//! This example expects a C-O-Si system containing `SiO2(quartz)`, `GAS`, and
//! `CO2`; the general beginner examples do not assume these names.

mod common;

use chemapp_rs::entities::stream::Stream;
use chemapp_rs::snapshot::STABLE_PHASE_ACTIVITY_THRESHOLD;
use chemapp_rs::{Calculator, ChemAppError, SnapshotOptions};

fn main() -> Result<(), ChemAppError> {
    let calculator: Calculator = common::calculator_from_env()?;

    let stream = Stream::new(&calculator, "ENTITY-DEMO", 298.15, 1.0)?;
    let quartz = calculator.engine().tqinp("SiO2(quartz)")?;
    stream.add_with_indices(quartz, 0, 1.0)?;
    stream.add_with_names("GAS", "CO2", 0.5)?;
    calculator.engine().tqstec("T", 0, 2500.0)?;
    calculator.engine().tqce(" ", 0, 0, (0.0, 0.0))?;

    let full = calculator.snapshot()?;
    let stable = calculator.snapshot_with_options(SnapshotOptions::stable_only())?;
    let live_table = calculator.table_string()?;
    assert_eq!(live_table, full.to_string());
    assert!(stable
        .phases()
        .iter()
        .all(|phase| phase.ac > STABLE_PHASE_ACTIVITY_THRESHOLD));
    println!("{live_table}");
    println!(
        "Captured {} phases in the full snapshot and {} stable phases",
        full.phases().len(),
        stable.phases().len()
    );
    for phase in calculator.phases()? {
        let model = phase.model()?;
        let sublattices = if model.trim().eq_ignore_ascii_case("PURE") {
            0
        } else {
            calculator.engine().tqnosl(phase.index())?
        };
        let species_count = phase.species()?.count();
        println!(
            "Species inspection: phase {} {} ({}) has {} sublattice(s) and {} species row(s)",
            phase.index(),
            phase.name()?,
            model,
            sublattices,
            species_count,
        );
    }
    let stream_snapshot = stream.snapshot()?;
    assert_eq!(stream.table_string()?, stream_snapshot.to_string());
    let mapping = calculator.mapping_temperature(2000.0, 2600.0, false)?;
    let stable_mapping = calculator.mapping_temperature_with_options(
        2000.0,
        2600.0,
        false,
        SnapshotOptions::stable_only(),
    )?;
    let listed_mapping = calculator.mapping_temperature_with_options(
        2000.0,
        2600.0,
        true,
        SnapshotOptions::stable_only(),
    )?;
    assert!(stable_mapping.iter().all(|snapshot| snapshot
        .phases()
        .iter()
        .all(|phase| phase.ac > STABLE_PHASE_ACTIVITY_THRESHOLD)));
    println!(
        "Captured {} full, {} stable-only, and {} listed mapping states",
        mapping.len(),
        stable_mapping.len(),
        listed_mapping.len()
    );
    stream.remove()?;
    Ok(())
}
