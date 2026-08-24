// entitiesdemo.rs
use std::path::PathBuf;

use chemapp_rs::entities::stream::Stream;
use chemapp_rs::snapshot::STABLE_PHASE_ACTIVITY_THRESHOLD;
use chemapp_rs::{Calculator, ChemAppError, SnapshotOptions};

fn main() -> Result<(), ChemAppError> {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    #[cfg(all(target_family = "windows", target_pointer_width = "32"))]
    let libpath = project_dir.join("windows").join("ca_vc_e_local.dll");
    #[cfg(all(target_family = "windows", target_pointer_width = "64"))]
    let libpath = project_dir.join("windows").join("ca_vc_e_x64.dll");
    #[cfg(target_family = "unix")]
    let libpath = project_dir.join("linux").join("libLChemAppS.so");
    let datafile_path = project_dir.join("data").join("cosi.dat");

    let libname = libpath.to_str().ok_or_else(|| {
        ChemAppError::OtherError("ChemApp library path is not valid UTF-8".to_owned())
    })?;
    let datafile = datafile_path.to_str().ok_or_else(|| {
        ChemAppError::OtherError("ChemApp data-file path is not valid UTF-8".to_owned())
    })?;
    let calculator = Calculator::from_library(libname, datafile)?;

    let stream = Stream::new(&calculator, "ENTITY-DEMO", 298.15, 1.0)?;
    let quartz = calculator.engine.tqinp("SiO2(quartz)")?;
    stream.add_with_indices(quartz, 0, 1.0)?;
    stream.add_with_names("GAS", "CO2", 0.5)?;
    calculator.engine.tqstec("T", 0, 2500.0)?;
    calculator.engine.tqce(" ", 0, 0, (0.0, 0.0))?;

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
    Ok(())
}
