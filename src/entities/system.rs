//! Fallible access to calculated whole-system properties.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::SystemSnapshot;

/// A live view of the current calculated ChemApp system state.
pub struct System<'a> {
    pub(crate) calculator: &'a Calculator,
}

impl<'a> System<'a> {
    /// Creates a live view of a calculator's whole-system results.
    pub fn new(calculator: &'a Calculator) -> Self {
        Self { calculator }
    }

    /// Copies the current native values into an engine-independent snapshot.
    pub fn snapshot(&self) -> Result<SystemSnapshot, ChemAppError> {
        SystemSnapshot::new(self)
    }

    /// Formats current system properties using the snapshot-compatible schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_system_table(self)
    }

    /// Returns temperature (`T`) in the active temperature unit.
    pub fn t(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("T", 0, 0)
    }

    /// Returns pressure (`P`) in the active pressure unit.
    pub fn p(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("P", 0, 0)
    }

    /// Returns total volume (`VT`) in the active volume unit.
    pub fn vt(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("VT", 0, 0)
    }

    /// Returns total amount (`A`) in the active amount unit.
    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("A", 0, 0)
    }

    /// Returns system heat capacity (`CP`) in the active unit.
    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("CP", 0, 0)
    }

    /// Returns system enthalpy (`H`) in the active unit.
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("H", 0, 0)
    }

    /// Returns system entropy (`S`) in the active unit.
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("S", 0, 0)
    }

    /// Returns system Gibbs energy (`G`) in the active unit.
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("G", 0, 0)
    }

    /// Returns system volume (`V`) in the active unit.
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("V", 0, 0)
    }
}
