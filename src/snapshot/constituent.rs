// chemapp_rs::snapshot::ConstituentSnapshot
//! A state snapshot of a phase constituent.

use crate::entities::constituent::Constituent;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

#[derive(Debug,Clone)]
pub struct ConstituentSnapshot {
	pub indexp : usize,
	pub index  : usize,
	pub status : String,
	pub name   : String,
	pub ia     : f64,
	pub a      : f64,
	pub ac     : f64,
	pub mu     : f64,
	pub h      : f64,
	pub s      : f64,
	pub g      : f64,
	pub cp     : f64,
	pub v      : f64,
	pub hm     : f64,
	pub sm     : f64,
	pub gm     : f64,
	pub cpm    : f64,
	pub vm     : f64,
}

impl ConstituentSnapshot {
	
	pub fn new(constituent: &Constituent)->Self {
		return Self {
			indexp : constituent.indexp,
			index  : constituent.index,
			status : constituent.status(),
			name   : constituent.name(),
			ia     : constituent.ia(),
			a      : constituent.a(),
			ac     : constituent.ac(),
			mu     : constituent.mu(),
			h      : constituent.h(),
			s      : constituent.s(),
			g      : constituent.g(),
			cp     : constituent.cp(),
			v      : constituent.v(),
			hm     : constituent.hm(),
			sm     : constituent.sm(),
			gm     : constituent.gm(),
			cpm    : constituent.cpm(),
			vm     : constituent.vm(),
		};
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/