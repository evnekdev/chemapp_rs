//! Fallible live access to a ChemApp phase constituent.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::ConstituentSnapshot;

#[derive(Debug)]
/// One phase constituent tied to the calculator's current native state.
pub struct Constituent<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) indexp: usize,
    pub(crate) index: usize,
}

impl<'a> Constituent<'a> {
    /// Creates a live view from one-based phase and constituent indices.
    pub fn new(calculator: &'a Calculator, indexp: usize, index: usize) -> Self {
        Self {
            calculator,
            indexp,
            index,
        }
    }

    /// Returns the one-based phase index.
    pub fn phase_index(&self) -> usize {
        self.indexp
    }
    /// Returns the one-based constituent index within the phase.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Copies the current constituent values into an owned snapshot.
    pub fn snapshot(&self) -> Result<ConstituentSnapshot, ChemAppError> {
        ConstituentSnapshot::new(self)
    }

    /// Formats this constituent using the shared live/snapshot table schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_constituent_table(self)
    }

    /// Reports whether both native indices exist in the loaded system.
    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        if self.indexp == 0 || self.indexp > self.calculator.engine.tqnop()? {
            return Ok(false);
        }
        Ok(self.index > 0 && self.index <= self.calculator.engine.tqnopc(self.indexp)?)
    }

    /// Returns the real-valued electric charge reported by ChemApp.
    pub fn charge(&self) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqchar(self.indexp, self.index)
    }

    /// Returns the constituent molar mass.
    pub fn wmass(&self) -> Result<f64, ChemAppError> {
        Ok(self.calculator.engine.tqstpc(self.indexp, self.index)?.1)
    }

    /// Returns stoichiometric coefficients in system-component order.
    pub fn stoic(&self) -> Result<Vec<f64>, ChemAppError> {
        Ok(self.calculator.engine.tqstpc(self.indexp, self.index)?.0)
    }

    /// Returns the current constituent status text.
    pub fn status(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgspc(self.indexp, self.index)
    }

    /// Returns the constituent name.
    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnpc(self.indexp, self.index)
    }

    /// Reports whether incoming material is allowed for this constituent.
    pub fn incoming_allowed(&self) -> Result<bool, ChemAppError> {
        self.calculator.engine.tqpcis(self.indexp, self.index)
    }

    fn result(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator
            .engine
            .tqgetr(option, self.indexp, self.index)
    }

    /// Returns the entered amount (`IA`).
    pub fn ia(&self) -> Result<f64, ChemAppError> {
        self.result("IA")
    }
    /// Returns the calculated amount (`A`).
    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.result("A")
    }
    /// Returns activity (`AC`).
    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.result("AC")
    }
    /// Returns chemical potential (`MU`).
    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.result("MU")
    }
    /// Returns enthalpy (`H`).
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.result("H")
    }
    /// Returns entropy (`S`).
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.result("S")
    }
    /// Returns Gibbs energy (`G`).
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.result("G")
    }
    /// Returns heat capacity (`CP`).
    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.result("CP")
    }
    /// Returns volume (`V`).
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.result("V")
    }
    /// Returns molar enthalpy (`HM`).
    pub fn hm(&self) -> Result<f64, ChemAppError> {
        self.result("HM")
    }
    /// Returns molar entropy (`SM`).
    pub fn sm(&self) -> Result<f64, ChemAppError> {
        self.result("SM")
    }
    /// Returns molar Gibbs energy (`GM`).
    pub fn gm(&self) -> Result<f64, ChemAppError> {
        self.result("GM")
    }
    /// Returns molar heat capacity (`CPM`).
    pub fn cpm(&self) -> Result<f64, ChemAppError> {
        self.result("CPM")
    }
    /// Returns molar volume (`VM`).
    pub fn vm(&self) -> Result<f64, ChemAppError> {
        self.result("VM")
    }
}
