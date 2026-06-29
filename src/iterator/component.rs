/// chemapp_rs::iterator::component.rs
use std::iter::{Filter, Map};
//use std::ops::{Range};
//use nalgebra::{DVector, SVector};
use crate::Calculator;

/********************************************************************************************************/
/********************************************************************************************************/

/// Facilitates iteration over indices of the system components
pub trait ComponentIterator where Self : Sized + Iterator<Item=usize>{
	
	fn components_valid(self, calculator: &Calculator)->Filter<Self,Box<dyn Fn(&usize)->bool>>{
		let ncomp = calculator.engine.tqnosc().unwrap();
		return self.filter(Box::new(move |idx: &usize| *idx > 0 && *idx <= ncomp));
	}
	
	fn components_names<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->String +'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgnsc(idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_wmass<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqstsc(idx).unwrap().1;
		};
		return self.map(Box::new(closure));
	}
	
	fn components_stoic<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->Vec<f64> + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqstsc(idx).unwrap().0;
		};
		return self.map(Box::new(closure));
	}
	
	fn components_ac<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("AC", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_mu<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("MU", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_a<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("A", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_x<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("X", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
}
