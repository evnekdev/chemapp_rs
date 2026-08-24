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
        let model = normalized_model(&calculator.engine.tqmodl(indexp)?);
        let mut identities = Vec::new();
        // The manual's retrieval example identifies the SUB* family as the
        // high-level sublattice-species surface. Non-applicable models are
        // empty by model semantics, not by swallowed native errors.
        if model.starts_with("SUB") {
            for sublattice in 1..=calculator.engine.tqnosl(indexp)? {
                for local_index in 1..=calculator.engine.tqnolc(indexp, sublattice)? {
                    identities.push(SpeciesRef {
                        sublattice,
                        local_index,
                    });
                }
            }
        }
        Ok(Self {
            calculator,
            indexp,
            identities: identities.into_iter(),
        })
    }
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
        let identities: Vec<_> = counts
            .iter()
            .enumerate()
            .flat_map(|(offset, count)| {
                (1..=*count).map(move |local_index| SpeciesRef {
                    sublattice: offset + 1,
                    local_index,
                })
            })
            .collect();
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
}
