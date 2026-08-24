//! Fallible live access to a ChemApp phase constituent.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::ConstituentSnapshot;

#[derive(Debug)]
pub struct Constituent<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) indexp: usize,
    pub(crate) index: usize,
}

impl<'a> Constituent<'a> {
    pub fn new(calculator: &'a Calculator, indexp: usize, index: usize) -> Self {
        Self {
            calculator,
            indexp,
            index,
        }
    }

    pub fn phase_index(&self) -> usize {
        self.indexp
    }
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn snapshot(&self) -> Result<ConstituentSnapshot, ChemAppError> {
        ConstituentSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_constituent_table(self)
    }

    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        if self.indexp == 0 || self.indexp > self.calculator.engine.tqnop()? {
            return Ok(false);
        }
        Ok(self.index > 0 && self.index <= self.calculator.engine.tqnopc(self.indexp)?)
    }

    pub fn charge(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqchar(self.indexp, self.index)
    }

    pub fn wmass(&self) -> Result<f64, ChemAppError> {
        Ok(self.calculator.engine.tqstpc(self.indexp, self.index)?.1)
    }

    pub fn stoic(&self) -> Result<Vec<f64>, ChemAppError> {
        Ok(self.calculator.engine.tqstpc(self.indexp, self.index)?.0)
    }

    pub fn status(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgspc(self.indexp, self.index)
    }

    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnpc(self.indexp, self.index)
    }

    pub fn incoming_allowed(&self) -> Result<bool, ChemAppError> {
        self.calculator.engine.tqpcis(self.indexp, self.index)
    }

    fn result(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator
            .engine
            .tqgetr(option, self.indexp, self.index)
    }

    pub fn ia(&self) -> Result<f64, ChemAppError> {
        self.result("IA")
    }
    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.result("A")
    }
    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.result("AC")
    }
    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.result("MU")
    }
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.result("H")
    }
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.result("S")
    }
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.result("G")
    }
    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.result("CP")
    }
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.result("V")
    }
    pub fn hm(&self) -> Result<f64, ChemAppError> {
        self.result("HM")
    }
    pub fn sm(&self) -> Result<f64, ChemAppError> {
        self.result("SM")
    }
    pub fn gm(&self) -> Result<f64, ChemAppError> {
        self.result("GM")
    }
    pub fn cpm(&self) -> Result<f64, ChemAppError> {
        self.result("CPM")
    }
    pub fn vm(&self) -> Result<f64, ChemAppError> {
        self.result("VM")
    }
}
