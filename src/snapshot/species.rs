// chemapp_rs::snapshot::species.rs
//! Chemical species snapshot

use crate::entities::species::Species;

#[derive(Debug,Clone)]
pub struct SpeciesSnapshot {
	pub indexp : usize,
	pub indexl : usize,
	pub indexs : usize,
	pub name   : String,
	pub x      : f64,
}

impl SpeciesSnapshot {
	
	pub fn new(species: &Species)->Self {
		todo!();
	}
	
}