// chemapp_rs::snapshot::system.rs
// Snapshot of global system properties.

use crate::entities::system::System;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

/// A snapshot of the global system properties
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
	pub t : f64,
	pub p : f64,
	pub vt: f64,
	pub a : f64,
}

impl SystemSnapshot {
	
	pub fn new(system: &System)->Self {
		return Self {
			t : system.t(),
			p : system.p(),
			vt: system.vt(),
			a : system.a(),
		};
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/