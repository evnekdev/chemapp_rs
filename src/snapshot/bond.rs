// chemapp_rs::snapshot::bond.rs
//! Chemical bond (quadruplet) snapshot.

use crate::entities::bond::Bond;

/// A state snapshot representing a quasichemical quadruplet.
#[derive(Debug,Clone)]
pub struct BondSnapshot {
	pub indexp   : usize,
	pub indexs1  : usize,
	pub indexs2  : usize,
	pub indexs3  : usize,
	pub indexs4  : usize,
	pub species1 : String,
	pub species2 : String,
	pub species3 : String,
	pub species4 : String,
	pub x        : f64,
}

impl BondSnapshot {
	
	/// create a new instance
	pub fn new(bond: &Bond)->Self {
		return Self {
			indexp   : bond.indexp,
			indexs1  : bond.indexs1,
			indexs2  : bond.indexs2,
			indexs3  : bond.indexs3,
			indexs4  : bond.indexs4,
			species1 : bond.species1().name(),
			species2 : bond.species2().name(),
			species3 : bond.species3().name(),
			species4 : bond.species4().name(),
			x        : bond.x(),
		};
	}
	
}