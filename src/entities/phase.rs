// phase.rs
//! `Phase` structure capturing the related functionality.
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::snapshot::PhaseSnapshot;
use crate::iterator::ConstituentIterator;
use crate::iterator::SpeciesIterator;
use crate::iterator::BondIterator;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

/// Phase representation
#[derive(Debug)]
pub struct Phase<'a> {
	calculator: &'a Calculator,
	pub(crate) index : usize,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> Phase<'a> {
	
	/// Make a new instance
	pub fn new(calculator: &'a Calculator, index: usize)->Self {
		return Self {
			calculator,
			index,
		};
	}
	
	/// take a snapshot of the current phase state
	pub fn snapshot(&self)->PhaseSnapshot {
		return PhaseSnapshot {
			index : self.index,
			status: self.status(),
			name  : self.name(),
			model : self.model(),
			a     : self.a(),
			ac    : self.ac(),
			mu    : self.mu(),
			h     : self.h(),
			s     : self.s(),
			g     : self.g(),
			cp    : self.cp(),
			v     : self.v(),
			hm    : self.hm(),
			sm    : self.sm(),
			gm    : self.gm(),
			cpm   : self.cpm(),
			vm    : self.vm(),
		};
	}
	
	/// Iterate over species in the phase
	pub fn species(&self)->SpeciesIterator<'_>{
		todo!();
	}
	
	/// Iterate over bonds (if any)
	pub fn bonds(&self)->BondIterator<'_>{
		todo!();
	}
	
	/// Iterate over phase constituents in the phase
	pub fn constituents(&self)->ConstituentIterator<'_>{
		todo!();
	}
	
	/// `true` if the phase index is between 1 and number of phases
	pub fn is_valid(&self)->bool {
		return self.index > 0 && self.index <= self.calculator.engine.tqnop().unwrap_or(0);
	}
	
	/// `true` if it is a stoichiometric phase
	pub fn is_stoic(&self)->bool {
		return self.model() == "PURE";
	}
	
	/// phase status ('ENTERED', 'DORMANT', 'ELIMINATED')
	pub fn status(&self)->String {
		return self.calculator.engine.tqgsp(self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// phase name
	pub fn name(&self)->String {
		return self.calculator.engine.tqgnp(self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// phase model
	pub fn model(&self)->String {
		return self.calculator.engine.tqmodl(self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// phase amount
	pub fn a(&self)->f64 {
		return self.calculator.engine.tqgetr("A", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// phase activity
	pub fn ac(&self)->f64 {
		return self.calculator.engine.tqgetr("AC", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// chemical potential
	pub fn mu(&self)->f64 {
		return self.calculator.engine.tqgetr("MU", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// enthalpy
	pub fn h(&self)->f64 {
		return self.calculator.engine.tqgetr("H", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// entropy
	pub fn s(&self)->f64 {
		return self.calculator.engine.tqgetr("S", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// gibbs energy
	pub fn g(&self)->f64 {
		return self.calculator.engine.tqgetr("G", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// heat capacity
	pub fn cp(&self)->f64 {
		return self.calculator.engine.tqgetr("CP", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// phase volume
	pub fn v(&self)->f64 {
		return self.calculator.engine.tqgetr("V", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// enthalpy per amount unit
	pub fn hm(&self)->f64 {
		return self.calculator.engine.tqgetr("HM", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// entropy per amount unit
	pub fn sm(&self)->f64 {
		return self.calculator.engine.tqgetr("SM", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// gibbs energy per amount unit
	pub fn gm(&self)->f64 {
		return self.calculator.engine.tqgetr("GM", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// heat capacity per amount unit
	pub fn cpm(&self)->f64 {
		return self.calculator.engine.tqgetr("CPM", self.index, 0).unwrap_or(f64::NAN);
	}
	
	/// phase volume per amount unit
	pub fn vm(&self)->f64 {
		return self.calculator.engine.tqgetr("VM", self.index, 0).unwrap_or(f64::NAN);
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

