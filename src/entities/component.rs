//! Fallible live access to a ChemApp system component.

use crate::calculator::Calculator;
use crate::entities::phase::Phase;
use crate::error::ChemAppError;
use crate::snapshot::SystemComponentSnapshot;

/// One-based ChemApp system-component identity tied to a live calculator.
pub struct SystemComponent<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) index: usize,
}

impl<'a> SystemComponent<'a> {
    /// Creates a live view for a one-based system-component index.
    pub fn new(calculator: &'a Calculator, index: usize) -> Self {
        Self { calculator, index }
    }

    /// Returns the one-based ChemApp component index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Copies the current component properties into an owned snapshot.
    pub fn snapshot(&self) -> Result<SystemComponentSnapshot, ChemAppError> {
        SystemComponentSnapshot::new(self)
    }

    /// Formats this component using the shared live/snapshot table schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_component_table(self)
    }

    /// Reports whether the index exists in the currently loaded system.
    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        Ok(self.index > 0 && self.index <= self.calculator.engine().tqnosc()?)
    }

    /// Returns the component name.
    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine().tqgnsc(self.index)
    }

    /// Returns the component molar mass.
    pub fn wmass(&self) -> Result<f64, ChemAppError> {
        Ok(self.calculator.engine().tqstsc(self.index)?.1)
    }

    /// Returns the component stoichiometry in system-component order.
    pub fn stoic(&self) -> Result<Vec<f64>, ChemAppError> {
        Ok(self.calculator.engine().tqstsc(self.index)?.0)
    }

    /// Returns the entered amount (`IA`) in the active amount unit.
    pub fn ia(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgetr("IA", 0, self.index)
    }

    /// Returns the calculated amount (`A`) in the active amount unit.
    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgetr("A", 0, self.index)
    }

    /// Returns the calculated activity (`AC`).
    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgetr("AC", 0, self.index)
    }

    /// Returns the chemical potential (`MU`) in the active energy unit.
    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgetr("MU", 0, self.index)
    }

    /// Returns the system mole fraction (`X`).
    pub fn x(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgetr("X", 0, self.index)
    }

    /// Returns this component's mole fraction (`XP`) in `phase`.
    pub fn xp(&self, phase: &Phase<'_>) -> Result<f64, ChemAppError> {
        self.calculator
            .engine()
            .tqgetr("XP", phase.index, self.index)
    }

    /// Returns this component's amount (`AP`) in `phase`.
    pub fn ap(&self, phase: &Phase<'_>) -> Result<f64, ChemAppError> {
        self.calculator
            .engine()
            .tqgetr("AP", phase.index, self.index)
    }
}
