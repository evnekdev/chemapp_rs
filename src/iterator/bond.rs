use crate::calculator::Calculator;
use crate::entities::bond::{normalized_model, Bond, BondKind};
use crate::entities::species::SpeciesRef;
use crate::error::ChemAppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BondIterationMode {
    Pair,
    Quadruplet,
    None,
}

/// Iterates canonical TQBOND pair or quadruplet identities for one phase.
pub struct BondIterator<'a> {
    calculator: &'a Calculator,
    indexp: usize,
    identities: std::vec::IntoIter<BondKind>,
}

impl<'a> BondIterator<'a> {
    /// Dispatches by TQMODL and enumerates only structurally applicable identities.
    pub fn new(calculator: &'a Calculator, indexp: usize) -> Result<Self, ChemAppError> {
        let mode = bond_iteration_mode(&calculator.engine.tqmodl(indexp)?);
        let identities = match mode {
            BondIterationMode::Pair => pair_identities(calculator.engine.tqnopc(indexp)?),
            BondIterationMode::Quadruplet => {
                if calculator.engine.tqnosl(indexp)? != 2 {
                    return Err(ChemAppError::OtherError(
                        "SUBG TQBOND enumeration requires exactly two sublattices".to_owned(),
                    ));
                }
                quadruplet_identities(
                    calculator.engine.tqnolc(indexp, 1)?,
                    calculator.engine.tqnolc(indexp, 2)?,
                )
            }
            BondIterationMode::None => Vec::new(),
        };
        Ok(Self {
            calculator,
            indexp,
            identities: identities.into_iter(),
        })
    }
}

impl<'a> Iterator for BondIterator<'a> {
    type Item = Bond<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.identities.next().map(|kind| Bond {
            calculator: self.calculator,
            indexp: self.indexp,
            kind,
        })
    }
}

pub(crate) fn bond_iteration_mode(model: &str) -> BondIterationMode {
    match normalized_model(model).as_str() {
        "SUBG" => BondIterationMode::Quadruplet,
        "QUAS" | "QSOL" => BondIterationMode::Pair,
        _ => BondIterationMode::None,
    }
}

/// Enumerates unordered pairs with replacement. The manual permits either
/// input order and does not exclude identical members, so A-A is present once
/// and A-B/B-A collapse to one identity.
pub(crate) fn pair_identities(count: usize) -> Vec<BondKind> {
    let mut values = Vec::new();
    for a in 1..=count {
        for b in a..=count {
            values.push(BondKind::Pair {
                constituent_a: a,
                constituent_b: b,
            });
        }
    }
    values
}

/// SUBG permits either order of the two members belonging to each
/// sublattice. Each within-sublattice pair is therefore an unordered
/// combination with replacement; no additional cross-sublattice equivalence
/// is assumed.
pub(crate) fn quadruplet_identities(first_count: usize, second_count: usize) -> Vec<BondKind> {
    let first_pairs: Vec<_> = (1..=first_count)
        .flat_map(|a| (a..=first_count).map(move |b| (a, b)))
        .collect();
    let second_pairs: Vec<_> = (1..=second_count)
        .flat_map(|a| (a..=second_count).map(move |b| (a, b)))
        .collect();
    let mut values = Vec::with_capacity(first_pairs.len() * second_pairs.len());
    for &(a, b) in &first_pairs {
        for &(c, d) in &second_pairs {
            values.push(BondKind::Quadruplet {
                species_a: SpeciesRef {
                    sublattice: 1,
                    local_index: a,
                },
                species_b: SpeciesRef {
                    sublattice: 1,
                    local_index: b,
                },
                species_c: SpeciesRef {
                    sublattice: 2,
                    local_index: c,
                },
                species_d: SpeciesRef {
                    sublattice: 2,
                    local_index: d,
                },
            });
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn model_dispatch_is_explicit() {
        assert_eq!(bond_iteration_mode("SUBG"), BondIterationMode::Quadruplet);
        assert_eq!(bond_iteration_mode("SUBGM"), BondIterationMode::Quadruplet);
        assert_eq!(bond_iteration_mode("QUAS"), BondIterationMode::Pair);
        assert_eq!(bond_iteration_mode("QSOL"), BondIterationMode::Pair);
        assert_eq!(bond_iteration_mode("PURE"), BondIterationMode::None);
        assert_eq!(bond_iteration_mode("IDMX"), BondIterationMode::None);
    }

    #[test]
    fn pair_generation_has_self_pairs_and_no_reversed_duplicates() {
        let values = pair_identities(3);
        assert_eq!(values.len(), 6);
        assert!(values.contains(&BondKind::Pair {
            constituent_a: 1,
            constituent_b: 1
        }));
        assert!(values.contains(&BondKind::Pair {
            constituent_a: 1,
            constituent_b: 3
        }));
        assert!(!values.contains(&BondKind::Pair {
            constituent_a: 3,
            constituent_b: 1
        }));
        assert_eq!(values.iter().collect::<HashSet<_>>().len(), values.len());
    }

    #[test]
    fn quadruplet_generation_removes_only_within_pair_permutations() {
        let values = quadruplet_identities(2, 2);
        assert_eq!(values.len(), 9);
        assert_eq!(values.iter().collect::<HashSet<_>>().len(), values.len());
        assert!(values.contains(&BondKind::Quadruplet {
            species_a: SpeciesRef {
                sublattice: 1,
                local_index: 1
            },
            species_b: SpeciesRef {
                sublattice: 1,
                local_index: 2
            },
            species_c: SpeciesRef {
                sublattice: 2,
                local_index: 1
            },
            species_d: SpeciesRef {
                sublattice: 2,
                local_index: 2
            },
        }));
    }
}
