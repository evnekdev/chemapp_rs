// entitiesdemo.rs
use std::path::{Path, PathBuf};

use chemapp_rs::entities::stream::Stream;
use chemapp_rs::snapshot::STABLE_PHASE_ACTIVITY_THRESHOLD;
use chemapp_rs::{Calculator, ChemAppError, SnapshotOptions};

/// Resolves an explicit vendor library before considering only binaries whose
/// architecture is actually checked into this repository. A source-modelled
/// Rust target is not evidence that a compatible ChemApp binary is bundled.
fn chemapp_library_path(project_dir: &Path) -> Result<PathBuf, ChemAppError> {
    if let Some(path) = std::env::var_os("CHEMAPP_LIBRARY") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    return Ok(project_dir.join("windows").join("ca_vc_e_local.dll"));
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Ok(project_dir.join("windows").join("ca_vc_e_x64.dll"));
    #[cfg(all(target_os = "linux", target_arch = "x86"))]
    return Ok(project_dir.join("linux").join("libLChemAppS.so"));

    #[allow(unreachable_code)]
    Err(ChemAppError::OtherError(format!(
        "no checked ChemApp native library is bundled for {}-{}; set CHEMAPP_LIBRARY to a compatible vendor library",
        std::env::consts::OS,
        std::env::consts::ARCH,
    )))
}

fn main() -> Result<(), ChemAppError> {
    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let libpath = chemapp_library_path(&project_dir)?;
    let datafile_path = std::env::var_os("CHEMAPP_DATAFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dir.join("data").join("cosi.dat"));

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
    for phase in calculator.phases()? {
        let model = phase.model()?;
        let sublattices = if model.trim().eq_ignore_ascii_case("PURE") {
            0
        } else {
            calculator.engine.tqnosl(phase.index())?
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
