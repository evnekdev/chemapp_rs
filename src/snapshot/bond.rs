//! Self-contained model-aware TQBOND snapshots.

use crate::entities::bond::{Bond, BondKind};
use crate::entities::constituent::Constituent;
use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
pub struct PairMemberSnapshot {
    pub constituent_index: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupletMemberSnapshot {
    pub identity: SpeciesRef,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BondSnapshotKind {
    Pair {
        constituent_a: PairMemberSnapshot,
        constituent_b: PairMemberSnapshot,
    },
    Quadruplet {
        species_a: QuadrupletMemberSnapshot,
        species_b: QuadrupletMemberSnapshot,
        species_c: QuadrupletMemberSnapshot,
        species_d: QuadrupletMemberSnapshot,
    },
}

/// Complete identity and fraction for one pair or quadruplet result.
#[derive(Debug, Clone, PartialEq)]
pub struct BondSnapshot {
    pub phase_index: usize,
    pub phase_name: String,
    pub model: String,
    pub kind: BondSnapshotKind,
    pub x: f64,
}

impl BondSnapshot {
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
