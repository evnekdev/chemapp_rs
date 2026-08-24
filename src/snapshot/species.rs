use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesSnapshot {
    pub phase_index: usize,
    pub phase_name: String,
    pub identity: SpeciesRef,
    pub name: String,
    pub x: f64,
}

impl SpeciesSnapshot {
    pub fn new(species: &Species<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            phase_index: species.phase_index(),
            phase_name: species.calculator.engine.tqgnp(species.phase_index())?,
            identity: species.identity(),
            name: species.name()?,
            x: species.x()?,
        })
    }
}
