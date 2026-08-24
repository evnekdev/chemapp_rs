use crate::calculator::Calculator;
use crate::entities::constituent::Constituent;
use crate::error::ChemAppError;

/// Iterates ordinary constituents of one phase in native order.
pub struct ConstituentIterator<'a> {
    calculator: &'a Calculator,
    next: usize,
    count: usize,
    indexp: usize,
}

impl<'a> ConstituentIterator<'a> {
    /// Queries the phase's current constituent count and creates the iterator.
    pub fn new(calculator: &'a Calculator, indexp: usize) -> Result<Self, ChemAppError> {
        Ok(Self {
            calculator,
            next: 1,
            count: calculator.engine.tqnopc(indexp)?,
            indexp,
        })
    }
}

impl<'a> Iterator for ConstituentIterator<'a> {
    type Item = Constituent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.count {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(Constituent::new(self.calculator, self.indexp, index))
    }
}
