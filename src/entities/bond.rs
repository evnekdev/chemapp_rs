//! Model-aware high-level representation of one calculated TQBOND result.

use crate::calculator::Calculator;
use crate::entities::constituent::Constituent;
use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;
use crate::snapshot::BondSnapshot;

/// TQBOND is model-dependent: QUAS/QSOL return pairs and SUBG/SUBQ return
/// quadruplets. A pair is never represented by fake third/fourth members.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BondKind {
    /// A canonical QUAS/QSOL pair of ordinary phase constituents.
    Pair {
        /// One-based first constituent index.
        constituent_a: usize,
        /// One-based second constituent index, not less than `constituent_a`.
        constituent_b: usize,
    },
    /// A canonical SUBG/SUBQ quadruplet with two species from each sublattice.
    Quadruplet {
        /// First member on sublattice one.
        species_a: SpeciesRef,
        /// Second member on sublattice one.
        species_b: SpeciesRef,
        /// First member on sublattice two.
        species_c: SpeciesRef,
        /// Second member on sublattice two.
        species_d: SpeciesRef,
    },
}

/// A live high-level TQBOND entity representing either one QUAS/QSOL pair
/// fraction or one SUBG/SUBQ quadruplet fraction.
pub struct Bond<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) indexp: usize,
    pub(crate) kind: BondKind,
}

impl<'a> Bond<'a> {
    /// Creates a canonical QUAS/QSOL pair; reversed members are reordered.
    pub fn new_pair(
        calculator: &'a Calculator,
        indexp: usize,
        constituent_a: usize,
        constituent_b: usize,
    ) -> Self {
        let (constituent_a, constituent_b) = canonical_pair(constituent_a, constituent_b);
        Self {
            calculator,
            indexp,
            kind: BondKind::Pair {
                constituent_a,
                constituent_b,
            },
        }
    }

    /// Creates a canonical SUBG/SUBQ quadruplet from local indices on each sublattice.
    pub fn new_quadruplet(
        calculator: &'a Calculator,
        indexp: usize,
        first_a: usize,
        first_b: usize,
        second_a: usize,
        second_b: usize,
    ) -> Self {
        let (first_a, first_b) = canonical_pair(first_a, first_b);
        let (second_a, second_b) = canonical_pair(second_a, second_b);
        Self {
            calculator,
            indexp,
            kind: BondKind::Quadruplet {
                species_a: SpeciesRef {
                    sublattice: 1,
                    local_index: first_a,
                },
                species_b: SpeciesRef {
                    sublattice: 1,
                    local_index: first_b,
                },
                species_c: SpeciesRef {
                    sublattice: 2,
                    local_index: second_a,
                },
                species_d: SpeciesRef {
                    sublattice: 2,
                    local_index: second_b,
                },
            },
        }
    }

    /// Returns the one-based parent phase index.
    pub fn phase_index(&self) -> usize {
        self.indexp
    }
    /// Returns the model-dependent pair or quadruplet identity.
    pub fn kind(&self) -> &BondKind {
        &self.kind
    }

    /// Copies the complete identity and current fraction into an owned snapshot.
    pub fn snapshot(&self) -> Result<BondSnapshot, ChemAppError> {
        BondSnapshot::new(self)
    }

    /// Formats this result using the shared live/snapshot bond schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_bond_table(self)
    }

    /// Validates the identity against the phase's current model and dimensions.
    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        if self.indexp == 0 || self.indexp > self.calculator.engine().tqnop()? {
            return Ok(false);
        }
        let model = normalized_model(&self.calculator.engine().tqmodl(self.indexp)?);
        match &self.kind {
            BondKind::Pair {
                constituent_a,
                constituent_b,
            } => {
                let count = self.calculator.engine().tqnopc(self.indexp)?;
                Ok(matches!(model.as_str(), "QUAS" | "QSOL")
                    && *constituent_a > 0
                    && constituent_a <= constituent_b
                    && *constituent_b <= count)
            }
            BondKind::Quadruplet {
                species_a,
                species_b,
                species_c,
                species_d,
            } => {
                if !matches!(model.as_str(), "SUBG" | "SUBQ")
                    || self.calculator.engine().tqnosl(self.indexp)? != 2
                {
                    return Ok(false);
                }
                let first_count = self.calculator.engine().tqnolc(self.indexp, 1)?;
                let second_count = self.calculator.engine().tqnolc(self.indexp, 2)?;
                Ok(species_a.sublattice == 1
                    && species_b.sublattice == 1
                    && species_c.sublattice == 2
                    && species_d.sublattice == 2
                    && species_a.local_index > 0
                    && species_a.local_index <= species_b.local_index
                    && species_b.local_index <= first_count
                    && species_c.local_index > 0
                    && species_c.local_index <= species_d.local_index
                    && species_d.local_index <= second_count)
            }
        }
    }

    /// Returns live constituent views for a pair, or `None` for a quadruplet.
    pub fn pair_members(&self) -> Option<(Constituent<'a>, Constituent<'a>)> {
        match self.kind {
            BondKind::Pair {
                constituent_a,
                constituent_b,
            } => Some((
                Constituent::new(self.calculator, self.indexp, constituent_a),
                Constituent::new(self.calculator, self.indexp, constituent_b),
            )),
            BondKind::Quadruplet { .. } => None,
        }
    }

    /// Returns live species views for a quadruplet, or `None` for a pair.
    pub fn quadruplet_members(&self) -> Option<[Species<'a>; 4]> {
        match self.kind {
            BondKind::Pair { .. } => None,
            BondKind::Quadruplet {
                species_a,
                species_b,
                species_c,
                species_d,
            } => Some([
                Species::new(
                    self.calculator,
                    self.indexp,
                    species_a.sublattice,
                    species_a.local_index,
                ),
                Species::new(
                    self.calculator,
                    self.indexp,
                    species_b.sublattice,
                    species_b.local_index,
                ),
                Species::new(
                    self.calculator,
                    self.indexp,
                    species_c.sublattice,
                    species_c.local_index,
                ),
                Species::new(
                    self.calculator,
                    self.indexp,
                    species_d.sublattice,
                    species_d.local_index,
                ),
            ]),
        }
    }

    /// Returns the pair or quadruplet fraction from TQBOND.
    pub fn x(&self) -> Result<f64, ChemAppError> {
        match self.kind {
            BondKind::Pair {
                constituent_a,
                constituent_b,
            } => {
                // The manual defines only INDEXA/INDEXB for QUAS/QSOL. Zeroes
                // are neutral placeholders for the unused INDEXC/INDEXD slots.
                self.calculator
                    .engine()
                    .tqbond(self.indexp, constituent_a, constituent_b, 0, 0)
            }
            BondKind::Quadruplet {
                species_a,
                species_b,
                species_c,
                species_d,
            } => {
                let first_count = self.calculator.engine().tqnolc(self.indexp, 1)?;
                self.calculator.engine().tqbond(
                    self.indexp,
                    species_a.local_index,
                    species_b.local_index,
                    quadruplet_native_index(first_count, species_c)?,
                    quadruplet_native_index(first_count, species_d)?,
                )
            }
        }
    }
}

pub(crate) fn normalized_model(model: &str) -> String {
    model.trim().to_ascii_uppercase().chars().take(4).collect()
}

pub(crate) fn canonical_pair(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Converts a local second-sublattice identity to the combined native index.
/// SUBG and runtime-verified SUBQ share this encoding; overflow is an error.
pub(crate) fn quadruplet_native_index(
    first_sublattice_count: usize,
    species: SpeciesRef,
) -> Result<usize, ChemAppError> {
    if species.sublattice != 2 || species.local_index == 0 {
        return Err(ChemAppError::OtherError(
            "SUBG/SUBQ offset conversion requires a positive second-sublattice identity".to_owned(),
        ));
    }
    first_sublattice_count
        .checked_add(species.local_index)
        .ok_or_else(|| ChemAppError::OtherError("SUBG/SUBQ native index overflow".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_identity_is_canonical() {
        assert_eq!(canonical_pair(2, 5), canonical_pair(5, 2));
        assert_eq!(canonical_pair(3, 3), (3, 3));
    }

    #[test]
    fn quadruplet_second_sublattice_uses_first_count_offset() {
        assert_eq!(
            quadruplet_native_index(
                4,
                SpeciesRef {
                    sublattice: 2,
                    local_index: 3
                }
            )
            .unwrap(),
            7
        );
    }

    #[test]
    fn model_codes_dispatch_by_their_four_character_base() {
        assert_eq!(normalized_model("SUBGM"), "SUBG");
        assert_eq!(normalized_model(" quas "), "QUAS");
    }
}
