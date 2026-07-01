// chemapp_rs::iterator::component.rs
//! `ComponentIterator` trait facilitating iteration and property retrieval for system components.
use std::iter::{Filter, Map};
use crate::Calculator;
use crate::entities::component::SystemComponent;

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

pub struct SystemComponentIterator<'a> {
	calculator : &'a Calculator,
	current : usize,
	ncomponents : usize,
}

impl<'a> SystemComponentIterator<'a> {
	
	pub fn new(calculator: &'a Calculator)->Self {
		let ncomponents = calculator.engine.tqnosc().unwrap_or(0);
		let current = 1;
		return Self {
			calculator,
			current,
			ncomponents,
		};
	}
	
}

impl<'a> Iterator for SystemComponentIterator<'a> {
	type Item = SystemComponent<'a>;
	
	fn next(&mut self)->Option<Self::Item>{
		if self.current > self.ncomponents {
			return None;
		}
		let current = self.current;
		self.current += 1;
		return Some(SystemComponent::new(self.calculator, current));
	}
	
}


/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/