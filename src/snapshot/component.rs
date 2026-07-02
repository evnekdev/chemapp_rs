
use crate::entities::component::SystemComponent;

/**********************************************************************************************************************/
/**********************************************************************************************************************/

#[derive(Debug,Clone)]
pub struct SystemComponentSnapshot {
	pub name : String,
	pub ia   : f64,
	pub a    : f64,
	pub ac   : f64,
	pub mu   : f64,
}

impl SystemComponentSnapshot {
	
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