// constituent.rs
//! An accessor structure `Constituent`
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
	
	/// charge of a phase constituent
	pub fn charge(&self)->i32 {
		return self.calculator.engine.tqchar(self.indexp, self.index).unwrap_or(0);
	}
	
	/// molar mass
	pub fn wmass(&self)->f64 {
		let ncomp = self.calculator.engine.tqnosc().unwrap_or(0);
		return self.calculator.engine.tqstpc(self.indexp, self.index).unwrap_or((vec![f64::NAN;ncomp],f64::NAN)).1;
	}
	
	/// stoichiometry vector
	pub fn stoic(&self)->Vec<f64> {
		let ncomp = self.calculator.engine.tqnosc().unwrap_or(0);
		return self.calculator.engine.tqstpc(self.indexp, self.index).unwrap_or((vec![f64::NAN;ncomp],f64::NAN)).0;
	}
	
	/// phase constituent status ('ENTERED', 'DORMANT', 'ELIMINATED')
	pub fn status(&self)->String {
		return self.calculator.engine.tqgspc(self.indexp, self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// phase constituent name
	pub fn name(&self)->String {
		return self.calculator.engine.tqgnpc(self.indexp, self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// `true` if the phase constituent can be used as an incoming species in `IA`
	pub fn incoming_allowed(&self)->bool {
		return self.calculator.engine.tqpcis(self.indexp, self.index).unwrap_or(false);
	}
	
	/// input amount
	pub fn ia(&self)->f64 {
		return self.calculator.engine.tqgetr("IA", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// amount
	pub fn a(&self)->f64 {
		return self.calculator.engine.tqgetr("A", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// activity
	pub fn ac(&self)->f64 {
		return self.calculator.engine.tqgetr("AC", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// chemical potential
	pub fn mu(&self)->f64 {
		return self.calculator.engine.tqgetr("MU", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// enthalpy
	pub fn h(&self)->f64 {
		return self.calculator.engine.tqgetr("H", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// entropy
	pub fn s(&self)->f64 {
		return self.calculator.engine.tqgetr("S", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// gibbs energy
	pub fn g(&self)->f64 {
		return self.calculator.engine.tqgetr("G", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// heat capacity
	pub fn cp(&self)->f64 {
		return self.calculator.engine.tqgetr("CP", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// volume
	pub fn v(&self)->f64 {
		return self.calculator.engine.tqgetr("V", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// enthalpy per amount unit
	pub fn hm(&self)->f64 {
		return self.calculator.engine.tqgetr("HM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// entropy per amount unit
	pub fn sm(&self)->f64 {
		return self.calculator.engine.tqgetr("SM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// gibbs energy per amount unit
	pub fn gm(&self)->f64 {
		return self.calculator.engine.tqgetr("GM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// heat capacity per amount unit
	pub fn cpm(&self)->f64 {
		return self.calculator.engine.tqgetr("CPM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
	/// volume per amount unit
	pub fn vm(&self)->f64 {
		return self.calculator.engine.tqgetr("VM", self.indexp, self.index).unwrap_or(f64::NAN);
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
