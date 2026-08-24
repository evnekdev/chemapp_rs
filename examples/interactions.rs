//! Prints every Gibbs and magnetic interaction in a loaded multicomponent DAT
//! model, including native TQLPAR text, name-resolved structure, live TQGPAR
//! values, and descriptor provenance.

use std::path::{Path, PathBuf};

use chemapp_rs::{Calculator, ChemAppError};

fn library_path(project: &Path) -> Result<PathBuf, ChemAppError> {
    if let Some(path) = std::env::var_os("CHEMAPP_LIBRARY") {
        return Ok(path.into());
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    return Ok(project.join("windows").join("ca_vc_e_local.dll"));
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Ok(project.join("windows").join("ca_vc_e_x64.dll"));
    #[cfg(all(target_os = "linux", target_arch = "x86"))]
    return Ok(project.join("linux").join("libLChemAppS.so"));

    #[allow(unreachable_code)]
    Err(ChemAppError::OtherError(
        "no checked bundled ChemApp library matches this target; set CHEMAPP_LIBRARY".to_owned(),
    ))
}

fn datafile_path(project: &Path) -> PathBuf {
    std::env::var_os("CHEMAPP_INTERACTION_DATAFILE")
        .or_else(|| std::env::var_os("CHEMAPP_DATAFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| project.join("data").join("cosi.dat"))
}

fn main() -> Result<(), ChemAppError> {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let library = library_path(&project)?;
    let datafile = datafile_path(&project);
    let calculator = Calculator::from_library(
        library
            .to_str()
            .ok_or_else(|| ChemAppError::OtherError("library path is not UTF-8".to_owned()))?,
        datafile
            .to_str()
            .ok_or_else(|| ChemAppError::OtherError("data-file path is not UTF-8".to_owned()))?,
    )?;

    eprintln!(
        "Descriptor recovery: not configured (native TQLPAR parsing only; use \
         interaction_report_with_recovery for a compatible ASCII DAT provider)."
    );

    for report in calculator.interaction_report()? {
        println!(
            "Phase {} [{}], model {}, {} sublattice(s): {} Gibbs, {} magnetic",
            report.phase_name,
            report.phase_index,
            report.model,
            report.sublattice_count,
            report.gibbs.len(),
            report.magnetic.len()
        );
        if !report.gibbs.is_empty() || !report.magnetic.is_empty() {
            println!("{}", report.table_string());
        }
    }
    Ok(())
}
