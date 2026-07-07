// chemapp_rs::system.rs
//! Global system properties.
use std::fmt;
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::snapshot::SystemSnapshot;

/**********************************************************************************************************************/
/**********************************************************************************************************************/
/// Accessor structure for retrieving global system properties
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
		return SystemSnapshot::new(self);
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
	
	pub fn print_header(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
		let tunit = self.calculator.engine.tqgsu("Temperature").unwrap_or("<NU>".to_owned());
		let punit = self.calculator.engine.tqgsu("Pressure").unwrap_or("<NU>".to_owned());
		let vunit = self.calculator.engine.tqgsu("Volume").unwrap_or("<NU>".to_owned());
		writeln!(f, "System properties, {:}, {:<12} {:}, {:<12} {:}, {:<12}", "T", &tunit, "P", &punit, "V", &vunit)?;
		return Ok(());
	}
	
	fn print_values(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
		let tval = self.t();
		let pval = self.p();
		let vval = self.vt();
		writeln!(f, "{:<18} {:<15} {:<15} {:<15}", "", &tval, &pval, &vval)?;
		return Ok(());
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> fmt::Debug for System<'a> {
	
	fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
		self.print_header(f)?;
		self.print_values(f)?;
		return Ok(());
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
