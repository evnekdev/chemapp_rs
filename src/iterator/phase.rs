use crate::calculator::Calculator;
use crate::entities::phase::Phase;
use crate::error::ChemAppError;

pub struct PhaseIterator<'a> {
    calculator: &'a Calculator,
    next: usize,
    count: usize,
}

impl<'a> PhaseIterator<'a> {
    pub fn new(calculator: &'a Calculator) -> Result<Self, ChemAppError> {
        Ok(Self {
            calculator,
            next: 1,
            count: calculator.engine.tqnop()?,
        })
    }
}

impl<'a> Iterator for PhaseIterator<'a> {
    type Item = Phase<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.count {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(Phase::new(self.calculator, index))
    }
}
