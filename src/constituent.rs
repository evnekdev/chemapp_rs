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
	
	pub fn snapshot(&self)->ConstituentSnapshot {
		todo!();
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
		todo!();
	}
	
	pub fn v(&self)->f64 {
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
	
	pub fn vm(&self)->f64 {
		todo!();
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/
