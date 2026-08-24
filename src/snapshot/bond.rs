//! Self-contained model-aware TQBOND snapshots.

use crate::entities::bond::{Bond, BondKind};
use crate::entities::constituent::Constituent;
use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
/// One ordinary phase constituent retained in a pair snapshot.
pub struct PairMemberSnapshot {
    /// One-based constituent index within the phase.
    pub constituent_index: usize,
    /// Constituent name captured from the live engine.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
/// One local sublattice species retained in a quadruplet snapshot.
pub struct QuadrupletMemberSnapshot {
    /// Conceptual local sublattice identity.
    pub identity: SpeciesRef,
    /// Species name captured from the live engine.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
/// Model-dependent identity of a captured TQBOND result.
pub enum BondSnapshotKind {
    /// A QUAS/QSOL pair of ordinary phase constituents.
    Pair {
        /// Canonical first pair member.
        constituent_a: PairMemberSnapshot,
        /// Canonical second pair member.
        constituent_b: PairMemberSnapshot,
    },
    /// A SUBG quadruplet with two members from each sublattice.
    Quadruplet {
        /// First member on sublattice one.
        species_a: QuadrupletMemberSnapshot,
        /// Second member on sublattice one.
        species_b: QuadrupletMemberSnapshot,
        /// First member on sublattice two.
        species_c: QuadrupletMemberSnapshot,
        /// Second member on sublattice two.
        species_d: QuadrupletMemberSnapshot,
    },
}

/// Complete identity and fraction for one pair or quadruplet result.
#[derive(Debug, Clone, PartialEq)]
pub struct BondSnapshot {
    /// One-based parent phase index.
    pub phase_index: usize,
    /// Parent phase name.
    pub phase_name: String,
    /// ChemApp phase-model identifier.
    pub model: String,
    /// Complete pair or quadruplet member identity and names.
    pub kind: BondSnapshotKind,
    /// Calculated pair or quadruplet fraction.
    pub x: f64,
}

impl BondSnapshot {
    /// Captures the complete model-aware identity and current TQBOND fraction.
    pub fn new(bond: &Bond<'_>) -> Result<Self, ChemAppError> {
        let kind = match bond.kind() {
            BondKind::Pair {
                constituent_a,
                constituent_b,
            } => {
                let a = Constituent::new(bond.calculator, bond.phase_index(), *constituent_a);
                let b = Constituent::new(bond.calculator, bond.phase_index(), *constituent_b);
                BondSnapshotKind::Pair {
                    constituent_a: PairMemberSnapshot {
                        constituent_index: *constituent_a,
                        name: a.name()?,
                    },
                    constituent_b: PairMemberSnapshot {
                        constituent_index: *constituent_b,
                        name: b.name()?,
                    },
                }
            }
            BondKind::Quadruplet {
                species_a,
                species_b,
                species_c,
                species_d,
            } => {
                let members = [species_a, species_b, species_c, species_d].map(|identity| {
                    Species::new(
                        bond.calculator,
                        bond.phase_index(),
                        identity.sublattice,
                        identity.local_index,
                    )
                });
                BondSnapshotKind::Quadruplet {
                    species_a: QuadrupletMemberSnapshot {
                        identity: *species_a,
                        name: members[0].name()?,
                    },
                    species_b: QuadrupletMemberSnapshot {
                        identity: *species_b,
                        name: members[1].name()?,
                    },
                    species_c: QuadrupletMemberSnapshot {
                        identity: *species_c,
                        name: members[2].name()?,
                    },
                    species_d: QuadrupletMemberSnapshot {
                        identity: *species_d,
                        name: members[3].name()?,
                    },
                }
            }
        };
        Ok(Self {
            phase_index: bond.phase_index(),
            phase_name: bond.calculator.engine.tqgnp(bond.phase_index())?,
            model: bond.calculator.engine.tqmodl(bond.phase_index())?,
            kind,
            x: bond.x()?,
        })
    }
}
