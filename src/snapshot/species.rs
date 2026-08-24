use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
/// Engine-independent copy of one sublattice species and site fraction.
pub struct SpeciesSnapshot {
    /// One-based parent phase index.
    pub phase_index: usize,
    /// Parent phase name.
    pub phase_name: String,
    /// Local one-based sublattice identity.
    pub identity: SpeciesRef,
    /// Species name.
    pub name: String,
    /// Calculated site fraction.
    pub x: f64,
}

impl SpeciesSnapshot {
    /// Captures identity and value from a live species.
    pub fn new(species: &Species<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            phase_index: species.phase_index(),
            phase_name: species.calculator.engine().tqgnp(species.phase_index())?,
            identity: species.identity(),
            name: species.name()?,
            x: species.x()?,
        })
    }
}
