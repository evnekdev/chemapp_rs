/// chemapp_rs::iterator::constituent.rs
use std::collections::{HashMap};
use std::iter::{Filter, Map};
use crate::Calculator;

/********************************************************************************************************/
/********************************************************************************************************/

/// Facilitates iteration over indices of phase constituents
pub trait ConstituentIterator where Self : Sized + Iterator<Item=(usize,usize)>{
	fn constituents_valid<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn FnMut(&(usize,usize))->bool + 'a>>{
		let nphases = calculator.engine.tqnop().unwrap();
		let mut cmap : HashMap<usize,usize> = HashMap::new();
		let closure = move |&(indexp, indexc): &(usize,usize)| {
			if indexp < 1 || indexp > nphases {return false;}
			match cmap.get(&indexp) {
				Some(&nconst) => {
					return indexc > 0 && indexc <= nconst;
				}
				None => {
					let nconst = calculator.engine.tqnopc(indexp).unwrap();
					cmap.insert(indexp, nconst);
					return indexc > 0 && indexc <= nconst;
				}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// Maps constituent indices to their corresponding names
	fn constituents_names<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->(String,String) + 'a>>{
		let closure = move |(indexp, indexc): (usize, usize)| {
			return (calculator.engine.tqgnp(indexp).unwrap(), calculator.engine.tqgnpc(indexp, indexc).unwrap());
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their amounts (after a tqce... routine was called)
	fn constituents_a<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("A", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their activities (after a tqce... routine was called)
	fn constituents_ac<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("AC", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their chemical potential values (after a tqce... routine was called)
	fn constituents_mu<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("MU", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their enthalpies (after a tqce... routine was called)
	fn constituents_h<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("H", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their Gibbs free energies (after a tqce... routine was called)
	fn constituents_g<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("G", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their entropies (after a tqce... routine was called)
	fn constituents_s<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("S", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their heat capacity values (after a tqce... routine was called)
	fn constituents_cp<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CP", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their volumes (after a tqce... routine was called)
	fn constituents_v<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("V", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their partial enthalpies (after a tqce... routine was called)
	fn constituents_hm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("HM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their partial Gibbs free energies (after a tqce... routine was called)
	fn constituents_gm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("GM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their partial entropies (after a tqce... routine was called)
	fn constituents_sm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("SM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their partial heat capacities (after a tqce... routine was called)
	fn constituents_cpm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CPM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps constituent indices to their partial volumes (after a tqce... routine was called)
	fn constituents_vm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("VM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
}