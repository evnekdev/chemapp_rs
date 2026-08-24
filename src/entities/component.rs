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
    pub fn new(calculator: &'a Calculator, index: usize) -> Self {
        Self { calculator, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn snapshot(&self) -> Result<SystemComponentSnapshot, ChemAppError> {
        SystemComponentSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_component_table(self)
    }

    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        Ok(self.index > 0 && self.index <= self.calculator.engine.tqnosc()?)
    }

    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnsc(self.index)
    }

    pub fn wmass(&self) -> Result<f64, ChemAppError> {
        Ok(self.calculator.engine.tqstsc(self.index)?.1)
    }

    pub fn stoic(&self) -> Result<Vec<f64>, ChemAppError> {
        Ok(self.calculator.engine.tqstsc(self.index)?.0)
    }

    pub fn ia(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("IA", 0, self.index)
    }

    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("A", 0, self.index)
    }

    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("AC", 0, self.index)
    }

    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("MU", 0, self.index)
    }

    pub fn x(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("X", 0, self.index)
    }

    pub fn xp(&self, phase: &Phase<'_>) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("XP", phase.index, self.index)
    }

    pub fn ap(&self, phase: &Phase<'_>) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr("AP", phase.index, self.index)
    }
}
