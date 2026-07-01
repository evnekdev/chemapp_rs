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
		todo!();
	}
	
	/// species name
	pub fn name(&self)->String {
		todo!();
	}
	
	pub fn sublattice(&self)->String {
		todo!();
	}
	
}


/**********************************************************************************************************************/
/**********************************************************************************************************************/
