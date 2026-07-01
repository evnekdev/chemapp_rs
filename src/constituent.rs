// constituent.rs
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::snapshot::ConstituentSnapshot;
 
/**********************************************************************************************************************/
/**********************************************************************************************************************/

#[derive(Debug)]
pub struct Constituent<'a> {
	calculator : &'a Calculator,
	pub(crate) indexp : usize,
	pub(crate) index  : usize,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> Constituent<'a> {
	
	/// Make a new instance
	pub fn new(calculator: &'a Calculator, indexp: usize, index: usize)->Self {
		return Self {
			calculator,
			indexp,
			index,
		};
	}
	
	/// make a snapshot of the current state
	pub fn snapshot(&self)->ConstituentSnapshot {
		return ConstituentSnapshot {
			indexp : self.indexp,
			index  : self.index,
			status : self.status(),
			name   : self.name(),
			ia     : self.ia(),
			a      : self.a(),
			ac     : self.ac(),
			mu     : self.mu(),
			h      : self.h(),
			s      : self.s(),
			g      : self.g(),
			cp     : self.cp(),
			v      : self.v(),
			hm     : self.hm(),
			sm     : self.sm(),
			gm     : self.gm(),
			cpm    : self.cpm(),
			vm     : self.vm(),
		};
	}
	
	
	pub fn is_valid(&self)->bool {
		todo!();
	}
	
	pub fn wmass(&self)->f64 {
		todo!();
	}
	
	pub fn stoic(&self)->DVector<f64> {
		todo!();
	}
	
	pub fn status(&self)->String {
		todo!();
	}
	
	pub fn name(&self)->String {
		todo!();
	}
	
	pub fn incoming_allowed(&self)->bool {
		todo!();
	}
	
	pub fn ia(&self)->f64 {
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
		return self.calculator.engine.tqgetr("CP", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn v(&self)->f64 {
		return self.calculator.engine.tqgetr("V", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn hm(&self)->f64 {
		return self.calculator.engine.tqgetr("HM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn sm(&self)->f64 {
		return self.calculator.engine.tqgetr("SM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn gm(&self)->f64 {
		return self.calculator.engine.tqgetr("GM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn cpm(&self)->f64 {
		return self.calculator.engine.tqgetr("CPM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	pub fn vm(&self)->f64 {
		return self.calculator.engine.tqgetr("VM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
