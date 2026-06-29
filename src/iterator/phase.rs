/// chemapp_rs::iterator::phase.rs
use std::iter::{Filter, Map, FlatMap};
use std::ops::Range;
use nalgebra::{DVector};
use crate::Calculator;

/********************************************************************************************************/
/********************************************************************************************************/
/// Facilitates iteration over phase indices
pub trait PhaseIterator where Self : Sized + Iterator<Item=usize>{
	/// for any input indices in the iterator, retains only those within 1..nphases range
	fn phases_valid(self, calculator: &Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool>>{
		let nphases = calculator.engine.tqnop().unwrap();
		return self.filter(Box::new(move |idx : &usize| *idx > 0 && *idx <= nphases));
	}
	/// retains only phases with AC = 1.00 (stable), filters out the rest (applicable only after a tqce... routine was called)
	fn phases_stable<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		return self.filter(Box::new(move |idx: &usize| calculator.engine.tqgetr("AC", *idx, 0).unwrap() > 0.9999));
	}
	/// maps phase indices to their corresponding model names
	fn phases_models<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->String + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqmodl(idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// retains only mixture (solution) phase indices
	fn solutions<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		let closure = move |idx: &usize| {
			let modl = calculator.engine.tqmodl(*idx).unwrap();
			//println!("modl = <{}>", &modl);
			match modl[0..4].as_ref() {
				"PURE" => {return false;}
				_ => {return true;}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// retains only `PURE` (stoichiometric compounds) indices
	fn compounds<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		let closure = move |idx: &usize| {
			let modl = calculator.engine.tqmodl(*idx).unwrap();
			match modl[0..4].as_ref() {
				"PURE" => {return true;}
				_ => {return false;}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// maps phase indices to their compositions in the input basis (applicable only after a tqce... routine was called)
	fn phases_compositions<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->DVector<f64> + 'a>>{
		let closure = move |idx: usize| {
			// TODO make sure it works correctly with other units!
			let ncomp = calculator.engine.tqnosc().unwrap();
			let mut xp : DVector<f64> = DVector::zeros(ncomp);
			for k in 0..ncomp {
				xp[k] = calculator.engine.tqgetr("XP", idx, k+1).unwrap();
			}
			let xe : DVector<f64> = calculator.transform.transform_init2final(&xp, false, false, true).column(0).into_owned();
			return xe;
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their relative activities in the system (applicable only after a tqce... routine was called)
	fn phases_ac<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("AC", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their amounts (applicable only after a tqce... routine was called)
	fn phases_a<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("A", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their chemical potentials (applicable only after a tqce... routine was called)
	fn phases_mu<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("MU", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their enthalpies (applicable only after a tqce... routine was called)
	fn phases_h<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("H", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their Gibbs free energies (applicable only after a tqce... routine was called)
	fn phases_g<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("G", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their entropies (applicable only after a tqce... routine was called)
	fn phases_s<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("S", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their heat capacities (applicable only after a tqce... routine was called)
	fn phases_cp<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CP", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their volumes (applicable only after a tqce... routine was called)
	fn phases_v<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("V", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their enthalpies per amount unit (applicable only after a tqce... routine was called)
	fn phases_hm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("HM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their Gibbs free energies per amount unit (applicable only after a tqce... routine was called)
	fn phases_gm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("GM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their entropies per amount unit (applicable only after a tqce... routine was called)
	fn phases_sm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("SM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their heat capacities per amount unit (applicable only after a tqce... routine was called)
	fn phases_cpm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CPM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their volumes per amount unit (applicable only after a tqce... routine was called)
	fn phases_vm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("VM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// maps phase indices to their names
	fn phases_names<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->String + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgnp(idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	/// retains only phase indices with `ENTERED` status
	fn phases_status_entered<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		let closure = move |idx: &usize| {
			let status = calculator.engine.tqgsp(*idx).unwrap();
			match status[0..4].as_ref(){
				"ENTE" => {
					return true;
					}
				_      => {return false;}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// retains only phase indices with `DORMANT` status
	fn phases_status_dormant<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		let closure = move |idx: &usize| {
			let status = calculator.engine.tqgsp(*idx).unwrap();
			match status[0..4].as_ref(){
				"DORM" => {
					return true;
					}
				_      => {return false;}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// retains only phase indices with `ELIMINATED` status
	fn phases_status_eliminated<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		let closure = move |idx: &usize| {
			let status = calculator.engine.tqgsp(*idx).unwrap();
			match status[0..4].as_ref(){
				"ELIM" => {return true;}
				_      => {return false;}
			}
		};
		return self.filter(Box::new(closure));
	}
	/// iterates over the phase constituent indices for each phase index in the iterator
	fn phases_constituents<'a>(self, calculator: &'a Calculator)->FlatMap<Self, Map<Range<usize>, Box<dyn FnMut(usize)->(usize,usize)>>, Box<dyn FnMut(usize)-> Map<Range<usize>, Box<dyn FnMut(usize)->(usize,usize)>> + 'a>>{
		let closure = move |indexp: usize| {
			return constituents_for(calculator, indexp);
		};
		return self.flat_map(Box::new(closure));
	}
	
}

fn constituents_for(calculator: &Calculator, indexp: usize)->Map<Range<usize>,Box<dyn FnMut(usize)->(usize,usize)>>{
	let ncons = calculator.engine.tqnopc(indexp).unwrap();
	//println!("indexp = {}, ncons = {}", indexp, ncons);
	return (1..ncons+1).map(Box::new(move |indexc: usize| {return (indexp,indexc);}));
}