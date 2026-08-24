use crate::calculator::Calculator;
use crate::entities::bond::normalized_model;
use crate::entities::species::{Species, SpeciesRef};
use crate::error::ChemAppError;

pub struct SpeciesIterator<'a> {
    calculator: &'a Calculator,
    indexp: usize,
    identities: std::vec::IntoIter<SpeciesRef>,
}

impl<'a> SpeciesIterator<'a> {
    pub fn new(calculator: &'a Calculator, indexp: usize) -> Result<Self, ChemAppError> {
        let model = calculator.engine.tqmodl(indexp)?;
        // TQMODL documents PURE as the result for a non-mixture phase. The
        // TQNOSL/TQNOLC contract applies to solution phases, which ChemApp
        // considers to have one or more sublattices. Do not infer that from a
        // textual model-family prefix: QUAS, QSOL, RKMP, and other mixture
        // models are not ruled out by their spelling.
        let identities = if has_sublattice_species(&model) {
            let counts = (1..=calculator.engine.tqnosl(indexp)?)
                .map(|sublattice| calculator.engine.tqnolc(indexp, sublattice))
                .collect::<Result<Vec<_>, _>>()?;
            flattened_species_identities(&counts)
        } else {
            Vec::new()
        };
        Ok(Self {
            calculator,
            indexp,
            identities: identities.into_iter(),
        })
    }
}

/// Returns whether ChemApp's documented sublattice query surface applies.
///
/// TQMODL returns `PURE` for non-mixture phases; all other documented model
/// codes represent mixture/solution phases for this purpose. This deliberately
/// is not a hand-maintained list of model names.
pub(crate) fn has_sublattice_species(model: &str) -> bool {
    normalized_model(model) != "PURE"
}

/// Flattens one-based local constituent indices without losing their
/// sublattice identity.
pub(crate) fn flattened_species_identities(counts: &[usize]) -> Vec<SpeciesRef> {
    counts
        .iter()
        .enumerate()
        .flat_map(|(offset, count)| {
            (1..=*count).map(move |local_index| SpeciesRef {
                sublattice: offset + 1,
                local_index,
            })
        })
        .collect()
}

impl<'a> Iterator for SpeciesIterator<'a> {
    type Item = Species<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.identities.next().map(|identity| {
            Species::new(
                self.calculator,
                self.indexp,
                identity.sublattice,
                identity.local_index,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattened_identity_preserves_sublattice_locality() {
        let counts = [2, 1, 3];
        let identities = flattened_species_identities(&counts);
        assert_eq!(identities.len(), 6);
        assert_eq!(
            identities[2],
            SpeciesRef {
                sublattice: 2,
                local_index: 1
            }
        );
        assert_eq!(
            identities[5],
            SpeciesRef {
                sublattice: 3,
                local_index: 3
            }
        );
    }

    #[test]
    fn applicability_uses_non_mixture_semantics_not_a_model_prefix() {
        assert!(!has_sublattice_species("PURE"));
        assert!(has_sublattice_species("RKMP"));
        assert!(has_sublattice_species("QUAS"));
        assert!(has_sublattice_species("QSOL"));
        assert!(has_sublattice_species("IDMX"));
    }

    #[test]
    fn zero_or_many_sublattices_preserve_one_based_identity() {
        assert!(flattened_species_identities(&[]).is_empty());
        assert!(flattened_species_identities(&[0]).is_empty());
        assert_eq!(
            flattened_species_identities(&[1, 0, 2]),
            vec![
                SpeciesRef {
                    sublattice: 1,
                    local_index: 1,
                },
                SpeciesRef {
                    sublattice: 3,
                    local_index: 1,
                },
                SpeciesRef {
                    sublattice: 3,
                    local_index: 2,
                },
            ]
        );
    }
}
