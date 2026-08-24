//! Shared environment handling for the maintained examples.

// Each example compiles this module independently and intentionally uses only
// the subset of helpers relevant to that example.
#![allow(dead_code)]

use std::path::PathBuf;

use chemapp_rs::{Calculator, ChemAppError};
use nalgebra::DVector;

pub fn required_path(variable: &str, purpose: &str) -> Result<PathBuf, ChemAppError> {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .ok_or_else(|| {
            ChemAppError::OtherError(format!(
                "set {variable} to the {purpose} before running this example"
            ))
        })
}

pub fn path_text<'a>(path: &'a std::path::Path, purpose: &str) -> Result<&'a str, ChemAppError> {
    path.to_str()
        .ok_or_else(|| ChemAppError::OtherError(format!("the {purpose} path is not valid UTF-8")))
}

pub fn calculator_from_env() -> Result<Calculator, ChemAppError> {
    let library = required_path("CHEMAPP_LIBRARY", "ChemApp DLL or shared library")?;
    let datafile = required_path("CHEMAPP_DATAFILE", "ChemApp DAT, BIN, or CST data-file")?;
    Calculator::from_library(
        path_text(&library, "ChemApp library")?,
        path_text(&datafile, "thermodynamic data-file")?,
    )
}

pub fn temperature() -> Result<f64, ChemAppError> {
    match std::env::var("CHEMAPP_TEMPERATURE") {
        Ok(value) => value.parse::<f64>().map_err(|error| {
            ChemAppError::OtherError(format!(
                "CHEMAPP_TEMPERATURE must be a number in the active ChemApp temperature unit: {error}"
            ))
        }),
        Err(std::env::VarError::NotPresent) => Ok(1000.0),
        Err(error) => Err(ChemAppError::OtherError(format!(
            "could not read CHEMAPP_TEMPERATURE: {error}"
        ))),
    }
}

pub fn pressure() -> Result<f64, ChemAppError> {
    match std::env::var("CHEMAPP_PRESSURE") {
        Ok(value) => value.parse::<f64>().map_err(|error| {
            ChemAppError::OtherError(format!(
                "CHEMAPP_PRESSURE must be a number in the active ChemApp pressure unit: {error}"
            ))
        }),
        Err(std::env::VarError::NotPresent) => Ok(1.0),
        Err(error) => Err(ChemAppError::OtherError(format!(
            "could not read CHEMAPP_PRESSURE: {error}"
        ))),
    }
}

/// Builds a non-degenerate demonstration composition in the loaded
/// system-component basis. Every component receives one unit; applications
/// must replace this with their scientifically meaningful composition.
pub fn unit_component_composition(calculator: &Calculator) -> Result<DVector<f64>, ChemAppError> {
    let count = calculator.engine().tqnosc()?;
    if count == 0 {
        return Err(ChemAppError::OtherError(
            "the loaded data-file contains no system components".to_owned(),
        ));
    }
    Ok(DVector::from_element(count, 1.0))
}
