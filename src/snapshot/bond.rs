// chemapp_rs::snapshot::bond.rs
//! Chemical bond (quadruplet) snapshot.

use crate::entities::bond::Bond;

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
	
	pub fn new(bond: &Bond)->Self {
		todo!();
	}
	
}