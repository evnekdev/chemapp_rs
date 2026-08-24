// entitiesdemo.rs
use std::path::PathBuf;

use chemapp_rs::{Calculator, ChemAppError};

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

    calculator.engine.tqsetc("T", 0, 0, 1200.0)?;
    calculator.engine.tqsetc("P", 0, 0, 1.0)?;
    calculator.engine.tqsetc("IA", 0, 1, 1.0)?;
    calculator.engine.tqsetc("IA", 0, 2, 0.2)?;
    calculator.engine.tqsetc("IA", 0, 3, 1.5)?;
    calculator.engine.tqcel(" ", 0, 0, (0.0, 0.0))?;

    calculator.print_system();
    calculator.print_components();
    Ok(())
}
