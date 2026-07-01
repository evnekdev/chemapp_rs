// chemapp_rs::system.rs
//! Global system properties.
use nalgebra::{DVector};

use crate::calculator::Calculator;

/**********************************************************************************************************************/
/**********************************************************************************************************************/
/// Accessor structure for retrieving global system properties
#[derive(Debug)]
pub struct System<'a> {
	calculator : &'a Calculator,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> System<'a> {
	
	/// create a new instance
	pub fn new(calculator: &'a Calculator)-> Self {
		return Self {calculator};
	}
	
	/// Create a snapshot instance
	pub fn snapshot(&self)->SystemSnapshot {
		return SystemSnapshot {
			t : self.t(),
			p : self.p(),
			vt: self.vt(),
			a : self.a(),
		};
	}
	
	/// system temperature
	pub fn t(&self)->f64 {
		return self.calculator.engine.tqgetr("T", 0, 0).unwrap_or(f64::NAN);
	}
	
	/// system pressure
	pub fn p(&self)->f64 {
		return self.calculator.engine.tqgetr("P", 0, 0).unwrap_or(f64::NAN);
	}
	
	/// total volume
	pub fn vt(&self)->f64 {
		return self.calculator.engine.tqgetr("VT", 0, 0).unwrap_or(f64::NAN);
	}
	
	/// total amount
	pub fn a(&self)->f64 {
		return self.calculator.engine.tqgetr("A", 0, 0).unwrap_or(f64::NAN);
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

/// A snapshot of the global system properties
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
	pub t : f64,
	pub p : f64,
	pub vt: f64,
	pub a : f64,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/