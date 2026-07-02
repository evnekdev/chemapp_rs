// chemapp_rs::bond.rs
//! MQM bond representation (not valid for all phases)

use crate::Calculator;
use crate::snapshot::BondSnapshot;
use crate::entities::species::Species;

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
		return BondSnapshot {
			indexp : self.indexp,
			indexs1: self.indexs1,
			indexs2: self.indexs2,
			indexs3: self.indexs3,
			indexs4: self.indexs4,
			species1: self.species1().name(),
			species2: self.species2().name(),
			species3: self.species3().name(),
			species4: self.species4().name(),
			x      : self.x(),
		};
	}
	
	/// `true` if all indices are valid and it is a correct model type (must be quasichemical)
	pub fn is_valid(&self)->bool {
		todo!();
	}
	
	/// first cation
	pub fn species1(&self)->Species {
		return Species::new(self.calculator, self.indexp, 1, self.indexs1);
	}
	
	/// second cation
	pub fn species2(&self)->Species {
		return Species::new(self.calculator, self.indexp, 1, self.indexs2);
	}
	
	/// first anion
	pub fn species3(&self)->Species {
		return Species::new(self.calculator, self.indexp, 2, self.indexs3);
	}
	
	/// second anion
	pub fn species4(&self)->Species {
		return Species::new(self.calculator, self.indexp, 2, self.indexs4);
	}
	
	/// molar/weight? fraction of the bond
	pub fn x(&self)->f64 {
		return self.calculator.engine.tqbond(self.indexp, self.indexs1, self.indexs2, self.indexs3, self.indexs4).unwrap_or(f64::NAN);
	}
	
}