//! Owned live ChemApp stream.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::StreamSnapshot;

/// A stream created by this value and removed from ChemApp when dropped.
pub struct Stream<'a> {
    pub(crate) calculator: &'a Calculator,
    name: String,
    temp: f64,
    pres: f64,
}

impl<'a> Stream<'a> {
    pub fn new(
        calculator: &'a Calculator,
        name: &str,
        temp: f64,
        pres: f64,
    ) -> Result<Self, ChemAppError> {
        calculator.engine.tqsttp(name, (temp, pres))?;
        Ok(Self {
            calculator,
            name: name.to_owned(),
            temp,
            pres,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn temperature(&self) -> f64 {
        self.temp
    }
    pub fn pressure(&self) -> f64 {
        self.pres
    }

    pub fn snapshot(&self) -> Result<StreamSnapshot, ChemAppError> {
        StreamSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_stream_table(self)
    }

    pub fn add_with_indices(
        &self,
        indexp: usize,
        indexc: usize,
        val: f64,
    ) -> Result<(), ChemAppError> {
        self.calculator
            .engine
            .tqstca(&self.name, indexp, indexc, val)
    }

    pub fn add_with_names(
        &self,
        phase: &str,
        constituent: &str,
        val: f64,
    ) -> Result<(), ChemAppError> {
        let indexp = self.calculator.engine.tqinp(phase)?;
        let indexc = self.calculator.engine.tqinpc(indexp, constituent)?;
        self.add_with_indices(indexp, indexc, val)
    }

    fn property(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqstxp(&self.name, option)
    }

    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.property("CP")
    }
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.property("H")
    }
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.property("S")
    }
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.property("G")
    }
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.property("V")
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        // Drop cannot report a native cleanup error. Explicit removal can be
        // added later if callers need observable cleanup failure.
        let _ = self.calculator.engine.tqstrm(&self.name);
    }
}
