// component.rs
//! `SystemComponent` structure capturing the related functionality.
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::phase::Phase;
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
	
	pub fn snapshot(&self)->SystemComponentSnapshot {
		todo!();
	}
	
	/// `true` if the inner index corresponds to an existing system component
	pub fn is_valid(&self)->bool {
		return self.index > 0 && self.index <= self.calculator.engine.tqnosc().unwrap_or(0);
	}
	
	pub fn name(&self)->String {
		todo!();
	}
	
	/// Molar mass
	pub fn wmass(&self)->f64 {
		todo!();
	}
	
	/// Stoichiometry vector
	pub fn stoic(&self)->DVector<f64>{
		todo!();
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
		todo!();
	}
	
	/// Amount in a phase
	pub fn ap(&self, phase: &Phase)->f64 {
		// TODO check calculator instance is the same.
		todo!();
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
