// chemapp_rs::snapshot::phase.rs
//! Phase snapshot : index, name + calculated properties + constituents + species + bonds

use crate::entities::phase::Phase;
use crate::snapshot::ConstituentSnapshot;
use crate::snapshot::SpeciesSnapshot;
use crate::snapshot::BondSnapshot;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

#[derive(Debug,Clone)]
pub struct PhaseSnapshot {
	pub index: usize,
	pub status : String,
	pub name : String,
	pub model: String,
	pub a    : f64,
	pub ac   : f64,
	pub mu   : f64,
	pub h    : f64,
	pub s    : f64,
	pub g    : f64,
	pub cp   : f64,
	pub v    : f64,
	pub hm   : f64,
	pub sm   : f64,
	pub gm   : f64,
	pub cpm  : f64,
	pub vm   : f64,
	pub constituents : Vec<ConstituentSnapshot>,
	pub species : Vec<SpeciesSnapshot>,
	pub bonds   : Vec<BondSnapshot>,
}

impl PhaseSnapshot {
	
	pub fn new(phase: &Phase)->Self {
		todo!();
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/