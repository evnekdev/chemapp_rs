//! Live sublattice-species identity and calculated site fraction.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::SpeciesSnapshot;

/// Conceptual one-based identity within one sublattice.
///
/// This deliberately does not expose TQBOND's combined second-sublattice
/// offset representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SpeciesRef {
    pub sublattice: usize,
    pub local_index: usize,
}

pub struct Species<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) indexp: usize,
    pub(crate) identity: SpeciesRef,
}

impl<'a> Species<'a> {
    pub fn new(
        calculator: &'a Calculator,
        indexp: usize,
        sublattice: usize,
        local_index: usize,
    ) -> Self {
        Self {
            calculator,
            indexp,
            identity: SpeciesRef {
                sublattice,
                local_index,
            },
        }
    }

    pub fn phase_index(&self) -> usize {
        self.indexp
    }
    pub fn identity(&self) -> SpeciesRef {
        self.identity
    }
    pub fn sublattice(&self) -> usize {
        self.identity.sublattice
    }
    pub fn local_index(&self) -> usize {
        self.identity.local_index
    }

    pub fn snapshot(&self) -> Result<SpeciesSnapshot, ChemAppError> {
        SpeciesSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_species_table(self)
    }

    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        if self.indexp == 0 || self.indexp > self.calculator.engine.tqnop()? {
            return Ok(false);
        }
        let number_of_sublattices = self.calculator.engine.tqnosl(self.indexp)?;
        if self.identity.sublattice == 0 || self.identity.sublattice > number_of_sublattices {
            return Ok(false);
        }
        Ok(self.identity.local_index > 0
            && self.identity.local_index
                <= self
                    .calculator
                    .engine
                    .tqnolc(self.indexp, self.identity.sublattice)?)
    }

    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnlc(
            self.indexp,
            self.identity.sublattice,
            self.identity.local_index,
        )
    }

    pub fn x(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgtlc(
            self.indexp,
            self.identity.sublattice,
            self.identity.local_index,
        )
    }
}
