// chemapp_rs::entities::stream.rs

use crate::calculator::Calculator;
use crate::error::ChemAppError;

pub struct Stream<'a> {
	calculator : &'a Calculator,
	name   : String,
	temp   : f64,
	pres   : f64,
}

impl<'a> Stream<'a> {
	
	/// create a new stream
	pub fn new(calculator: &'a Calculator, name: &str, temp: f64, pres : f64)->Result<Self,ChemAppError>{
		/// TODO - what about units?
		calculator.engine.tqsttp(name, (temp, pres))?;
		return Ok(Self {
			calculator,
			name : name.to_owned(),
			temp,
			pres,
		});
	}
	
	/// Add incoming amount of a phase constituent, use ChemApp indices for indentification.
	pub fn add_with_indices(&self, indexp: usize, indexc: usize, val: f64)->Result<(),ChemAppError>{
		return self.calculator.engine.tqstca(&self.name, indexp, indexc, val);
	}
	
	/// Add incoming amount of a phase constituent, use names for identification.
	pub fn add_with_names(&self, phase: &str, constituent: &str, val: f64)->Result<(),ChemAppError>{
		let indexp = self.calculator.engine.tqinp(phase)?;
		let indexc = self.calculator.engine.tqinpc(indexp, constituent)?;
		return self.calculator.engine.tqstca(&self.name, indexp, indexc, val);
	}
	
	/// heat capacity [current energy unit]/K
	pub fn cp(&self)->f64 {
		return self.calculator.engine.tqstxp(&self.name, "CP").unwrap_or(f64::NAN);
	}
	
	/// enthalpy [current energy unit]
	pub fn h(&self)->f64 {
		return self.calculator.engine.tqstxp(&self.name, "H").unwrap_or(f64::NAN);
	}
	
	/// entropy [current energy unit]/K
	pub fn s(&self)->f64 {
		return self.calculator.engine.tqstxp(&self.name, "S").unwrap_or(f64::NAN);
	}
	
	/// gibbs energy [current energy unit]
	pub fn g(&self)->f64 {
		return self.calculator.engine.tqstxp(&self.name, "G").unwrap_or(f64::NAN);
	}
	
	/// volume [current volume unit]
	pub fn v(&self)->f64 {
		return self.calculator.engine.tqstxp(&self.name, "V").unwrap_or(f64::NAN);
	}
	
}

impl<'a> Stream<'a> {
	
	fn drop(&mut self){
		self.calculator.engine.tqstrm(&self.name);
	}
	
}