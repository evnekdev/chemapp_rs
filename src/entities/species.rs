//! Live sublattice-species identity and calculated site fraction.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::iterator::species::has_sublattice_species;
use crate::snapshot::SpeciesSnapshot;

/// Conceptual one-based identity within one sublattice.
///
/// This deliberately does not expose TQBOND's combined second-sublattice
/// offset representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SpeciesRef {
    /// One-based sublattice index.
    pub sublattice: usize,
    /// One-based constituent index local to that sublattice.
    pub local_index: usize,
}

/// One live sublattice-species view tied to the calculator's native state.
pub struct Species<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) indexp: usize,
    pub(crate) identity: SpeciesRef,
}

impl<'a> Species<'a> {
    /// Creates a live species view from one-based phase and local identities.
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

    /// Returns the one-based parent phase index.
    pub fn phase_index(&self) -> usize {
        self.indexp
    }
    /// Returns the conceptual local sublattice identity.
    pub fn identity(&self) -> SpeciesRef {
        self.identity
    }
    /// Returns the one-based sublattice index.
    pub fn sublattice(&self) -> usize {
        self.identity.sublattice
    }
    /// Returns the one-based constituent index within its sublattice.
    pub fn local_index(&self) -> usize {
        self.identity.local_index
    }

    /// Copies the current species values into an owned snapshot.
    pub fn snapshot(&self) -> Result<SpeciesSnapshot, ChemAppError> {
        SpeciesSnapshot::new(self)
    }

    /// Formats this species using the shared live/snapshot table schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_species_table(self)
    }

    /// Reports whether this species identity is valid for the phase model.
    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        if self.indexp == 0 || self.indexp > self.calculator.engine().tqnop()? {
            return Ok(false);
        }
        if !has_sublattice_species(&self.calculator.engine().tqmodl(self.indexp)?) {
            return Ok(false);
        }
        let number_of_sublattices = self.calculator.engine().tqnosl(self.indexp)?;
        if self.identity.sublattice == 0 || self.identity.sublattice > number_of_sublattices {
            return Ok(false);
        }
        Ok(self.identity.local_index > 0
            && self.identity.local_index
                <= self
                    .calculator
                    .engine()
                    .tqnolc(self.indexp, self.identity.sublattice)?)
    }

    /// Returns the sublattice-species name.
    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine().tqgnlc(
            self.indexp,
            self.identity.sublattice,
            self.identity.local_index,
        )
    }

    /// Returns the calculated sublattice site fraction.
    pub fn x(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine().tqgtlc(
            self.indexp,
            self.identity.sublattice,
            self.identity.local_index,
        )
    }
}
