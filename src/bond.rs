// chemapp_rs::bond.rs

use crate::Calculator;
use crate::snapshot::BondSnapshot;

/// A structure representing a bond in an MQM model.
pub struct Bond<'a> {
	calculator : &'a Calculator,
	indexp   : usize,
	indexs1  : usize,
	indexs2  : usize,
	indexs3  : usize,
	indexs4  : usize,
}

impl<'a> Bond<'a> {
	
	/// create a new instance
	pub fn new(calculator: &'a Calculator, indexp: usize, indexs1: usize, indexs2: usize, indexs3: usize, indexs4: usize)->Self {
		return Self {
			calculator,
			indexp,
			indexs1,
			indexs2,
			indexs3,
			indexs4,
		};
	}
	
	/// make a snapshot of the current state
	pub fn snapshot(&self)->BondSnapshot {
		todo!();
	}
	
	/// `true` if all indices are valid and it is a correct model type (must be quasichemical)
	pub fn is_valid(&self)->bool {
		todo!();
	}
	
	/// molar/weight? fraction of the bond
	pub fn x(&self)->f64 {
		return self.calculator.engine.tqbond(self.indexp, self.indexs1, self.indexs2, self.indexs3, self.indexs4).unwrap_or(f64::NAN);
	}
	
}