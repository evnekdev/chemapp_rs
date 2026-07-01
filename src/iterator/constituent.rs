// chemapp_rs::iterator::constituent.rs
//! `ConstituentIterator` trait facilitating iteration and property retrieval for phase constituents.
use std::collections::{HashMap};
use std::iter::{Filter, Map};
use crate::Calculator;
use crate::entities::constituent::Constituent;

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

pub struct ConstituentIterator<'a> {
	calculator    : &'a Calculator,
	current       : usize,
	nconstituents : usize,
	indexp        : usize,
}

impl<'a> ConstituentIterator<'a> {
	
	pub fn new(calculator: &'a Calculator, indexp: usize)->Self {
		let nconstituents = calculator.engine.tqnopc(indexp).unwrap_or(0);
		let current = 1;
		return Self {
			calculator,
			current,
			nconstituents,
			indexp,
		};
	}
	
}

impl<'a> Iterator for ConstituentIterator<'a> {
	type Item = Constituent<'a>;
	
	fn next(&mut self)->Option<Self::Item> {
		if self.current > self.nconstituents {
			return None;
		}
		let current = self.current;
		self.current += 1;
		return Some(Constituent::new(self.calculator, self.indexp, current));
	}
	
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/