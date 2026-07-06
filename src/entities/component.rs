// component.rs
//! `SystemComponent` structure capturing the related functionality.
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::entities::phase::Phase;
use crate::snapshot::SystemComponentSnapshot;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

/// System component representation
#[derive(Debug)]
pub struct SystemComponent<'a> {
	calculator : &'a Calculator,
	pub(crate) index : usize,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> SystemComponent<'a> {
	
	/// Instantiation
	pub fn new(calculator: &'a Calculator, index: usize)->Self {
		return Self {
			calculator,
			index,
		};
	}
	
	/// copy the properties into a snapshot structure
	pub fn snapshot(&self)->SystemComponentSnapshot {
		return SystemComponentSnapshot::new(self);
	}
	
	/// `true` if the inner index corresponds to an existing system component
	pub fn is_valid(&self)->bool {
		return self.index > 0 && self.index <= self.calculator.engine.tqnosc().unwrap_or(0);
	}
	
	/// system component name
	pub fn name(&self)->String {
		return self.calculator.engine.tqgnsc(self.index).unwrap_or("<NONE>".to_owned());
	}
	
	/// Molar mass
	pub fn wmass(&self)->f64 {
		let ncomp = self.calculator.engine.tqnosc().unwrap_or(0);
		return self.calculator.engine.tqstsc(self.index).unwrap_or((vec![f64::NAN;ncomp],f64::NAN)).1;
	}
	
	/// Stoichiometry vector
	pub fn stoic(&self)->Vec<f64>{
		let ncomp = self.calculator.engine.tqnosc().unwrap_or(0);
		return self.calculator.engine.tqstsc(self.index).unwrap_or((vec![f64::NAN;ncomp],f64::NAN)).0;
	}
	
	/// Input amount
	pub fn ia(&self)->f64 {
		return self.calculator.engine.tqgetr("IA", 0, self.index).unwrap_or(f64::NAN);
	}
	
	/// Amount
	pub fn a(&self)->f64 {
		return self.calculator.engine.tqgetr("A", 0, self.index).unwrap_or(f64::NAN);
	}
	
	/// Activity
	pub fn ac(&self)->f64 {
		return self.calculator.engine.tqgetr("AC", 0, self.index).unwrap_or(f64::NAN);
	}
	
	/// Chemical potential
	pub fn mu(&self)->f64 {
		return self.calculator.engine.tqgetr("MU", 0, self.index).unwrap_or(f64::NAN);
	}
	
	/// Molar/weight fraction in the system
	pub fn x(&self)->f64 {
		return self.calculator.engine.tqgetr("X", 0, self.index).unwrap_or(f64::NAN);
	}
	
	/// Molar/weight fraction in a phase
	pub fn xp(&self, phase: &Phase)->f64 {
		// TODO check calculator instance is the same.
		return self.calculator.engine.tqgetr("XP", phase.index, self.index).unwrap_or(f64::NAN);
	}
	
	/// Amount in a phase
	pub fn ap(&self, phase: &Phase)->f64 {
		// TODO check calculator instance is the same.
		return self.calculator.engine.tqgetr("AP", phase.index, self.index).unwrap_or(f64::NAN);
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
