// phase.rs
//! `Phase` structure capturing the related functionality.
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::snapshot::PhaseSnapshot;

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
	
	pub fn new(calculator: &'a Calculator, index: usize)->Self {
		return Self {
			calculator,
			index,
		};
	}
	
	pub fn snapshot(&self)->PhaseSnapshot {
		return PhaseSnapshot {
			index : self.index,
			name  : self.name(),
			model : self.model(),
			a     : self.a(),
			ac    : self.ac(),
			mu    : self.mu(),
			h     : self.h(),
			s     : self.s(),
			g     : self.g(),
			cp    : self.cp(),
			hm    : self.hm(),
			sm    : self.sm(),
			gm    : self.gm(),
			cpm   : self.cpm(),
		};
	}
	
	pub fn is_valid(&self)->bool {
		todo!();
	}
	
	pub fn is_stoic(&self)->bool {
		todo!();
	}
	
	pub fn name(&self)->String {
		todo!();
	}
	
	pub fn model(&self)->String {
		todo!();
	}
	
	pub fn a(&self)->f64 {
		todo!();
	}
	
	pub fn ac(&self)->f64 {
		todo!();
	}
	
	pub fn mu(&self)->f64 {
		todo!();
	}
	
	pub fn h(&self)->f64 {
		todo!();
	}
	
	pub fn s(&self)->f64 {
		todo!();
	}
	
	pub fn g(&self)->f64 {
		todo!();
	}
	
	pub fn cp(&self)->f64 {
		todo!();
	}
	
	pub fn hm(&self)->f64 {
		todo!();
	}
	
	pub fn sm(&self)->f64 {
		todo!();
	}
	
	pub fn gm(&self)->f64 {
		todo!();
	}
	
	pub fn cpm(&self)->f64 {
		todo!();
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

