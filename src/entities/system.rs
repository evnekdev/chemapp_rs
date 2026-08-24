//! Fallible access to calculated whole-system properties.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::SystemSnapshot;

/// A live view of the current calculated ChemApp system state.
pub struct System<'a> {
    pub(crate) calculator: &'a Calculator,
}

impl<'a> System<'a> {
    pub fn new(calculator: &'a Calculator) -> Self {
        Self { calculator }
    }

    /// Copies the current native values into an engine-independent snapshot.
    pub fn snapshot(&self) -> Result<SystemSnapshot, ChemAppError> {
        SystemSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_system_table(self)
    }

    pub fn t(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("T", 0, 0)
    }

    pub fn p(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("P", 0, 0)
    }

    pub fn vt(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("VT", 0, 0)
    }

    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("A", 0, 0)
    }

    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("CP", 0, 0)
    }

    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("H", 0, 0)
    }

    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("S", 0, 0)
    }

    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("G", 0, 0)
    }

    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("V", 0, 0)
    }
}
