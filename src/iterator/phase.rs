// chemapp_rs::iterator::phase.rs
//! `PhaseIterator` trait facilitating iteration and property retrieval for phases.
use std::iter::{Filter, Map, FlatMap};
use std::ops::Range;
use nalgebra::{DVector};
use crate::Calculator;
use crate::entities::phase::Phase;

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

pub struct PhaseIterator<'a> {
	calculator : &'a Calculator,
	current : usize,
	nphases : usize,
}

impl<'a> PhaseIterator<'a> {
	
	pub fn new(calculator : &'a Calculator)->Self {
		let nphases = calculator.engine.tqnop().unwrap_or(0);
		let current = 1;
		return Self {
			calculator,
			current,
			nphases,
		};
	}
	
}

impl<'a> Iterator for PhaseIterator<'a> {
	type Item = Phase<'a>;
	
	fn next(&mut self)->Option<Self::Item>{
		if self.current > self.nphases {
			return None;
		}
		let current = self.current;
		self.current += 1;
		return Some(Phase::new(self.calculator, current));
	}
	
}


/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/