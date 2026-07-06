// chemapp_rs::snapshot::component.rs
//! System component snapshot : index, name + calculated properties

use crate::entities::component::SystemComponent;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

/// A state snapshot of a system component
#[derive(Debug,Clone)]
pub struct SystemComponentSnapshot {
	pub name : String,
	pub ia   : f64,
	pub a    : f64,
	pub ac   : f64,
	pub mu   : f64,
}

impl SystemComponentSnapshot {
	
	/// create a new instance
	pub fn new(component: &SystemComponent)->Self {
		return SystemComponentSnapshot {
			name : component.name(),
			ia   : component.ia(),
			a    : component.a(),
			ac   : component.ac(),
			mu   : component.mu(),
		};
	}
	
}

/**********************************************************************************************************************/
/**********************************************************************************************************************/