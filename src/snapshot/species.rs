// chemapp_rs::snapshot::species.rs
//! Chemical species snapshot

use crate::entities::species::Species;

/// A state snapshot of a sublattice species.
#[derive(Debug,Clone)]
pub struct SpeciesSnapshot {
	pub indexp : usize,
	pub indexl : usize,
	pub indexs : usize,
	pub name   : String,
	pub x      : f64,
}

impl SpeciesSnapshot {
	
	/// create a new instance
	pub fn new(species: &Species)->Self {
		return Self {
			indexp : species.indexp,
			indexl : species.indexl,
			indexs : species.indexs,
			name   : species.name(),
			x      : species.x(),
		};
	}
	
}