// chemapp_rs::species.rs
//! An accessor structure `Species`
use nalgebra::{DVector};

use crate::calculator::Calculator;
use crate::snapshot::SpeciesSnapshot;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

pub struct Species<'a> {
	calculator: &'a Calculator,
	pub(crate) indexp : usize,
	pub(crate) indexl : usize,
	pub(crate) indexs : usize,
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/

impl<'a> Species<'a> {
	
	/// make a new instance
	pub fn new(calculator: &'a Calculator, indexp: usize, indexl: usize, indexs: usize)->Self {
		return Self {
			calculator,
			indexp,
			indexl,
			indexs,
		};
	}
	
	/// make a snapshot of the current state
	pub fn snapshot(&self)->SpeciesSnapshot {
		return SpeciesSnapshot {
			indexp : self.indexp,
			indexl : self.indexl,
			indexs : self.indexs,
			name   : self.name(),
			x      : self.x(),
		};
	}
	
	/// species name
	pub fn name(&self)->String {
		return self.calculator.engine.tqgnlc(self.indexp, self.indexl, self.indexs).unwrap_or("<NONE>".to_owned());
	}
	
	/// sublattice index
	pub fn sublattice(&self)->usize {
		return self.indexl;
	}
	
	/// calculated equilibrium sublattice site fraction
	pub fn x(&self)->f64 {
		return self.calculator.engine.tqgtlc(self.indexp, self.indexl, self.indexs).unwrap_or(f64::NAN);
	}
	
}


/**********************************************************************************************************************/
/**********************************************************************************************************************/
