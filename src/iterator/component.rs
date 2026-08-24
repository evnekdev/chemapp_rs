use crate::calculator::Calculator;
use crate::entities::component::SystemComponent;
use crate::error::ChemAppError;

pub struct SystemComponentIterator<'a> {
    calculator: &'a Calculator,
    next: usize,
    count: usize,
}

impl<'a> SystemComponentIterator<'a> {
    pub fn new(calculator: &'a Calculator) -> Result<Self, ChemAppError> {
        Ok(Self {
            calculator,
            next: 1,
            count: calculator.engine.tqnosc()?,
        })
    }
}

impl<'a> Iterator for SystemComponentIterator<'a> {
    type Item = SystemComponent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.count {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(SystemComponent::new(self.calculator, index))
    }
}
