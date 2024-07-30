// chemapp::calculator.rs

/// A high level module for easy operations on ChemApp library

use std::cmp::{Ordering};
use std::path::Path;
use std::ffi::OsStr;
use std::collections::{HashMap};
use std::iter::{Filter, Map, FlatMap};
use std::ops::{Range};
use nalgebra::{DVector, SVector, Matrix, Dim, DimName, U1, Storage};
//use serde::{Serialize,Serializer};

use crate::{Engine, error::{ChemAppError}};
use chemformula::{Transform};


/********************************************************************************************************/
/********************************************************************************************************/
//#[derive(Serialize, Deserialize)]
pub struct Calculator {
	pub engine: Engine,
	pub file: String,
	pub number_isothermal: usize,
	pub number_target_t: usize,
	pub transform: Transform,
}

fn get_extension_from_filename(filename: &str)->Option<String>{
	return Path::new(filename).extension().and_then(|s| OsStr::to_str(s).and_then(|s| Some(s.to_lowercase())));
}

impl Default for Calculator {
	fn default()->Calculator{
		return Calculator{
			engine: Engine::default(),
			file: r"".to_string(),
			number_isothermal: 0,
			number_target_t: 0,
			transform: Transform::default(),
		};
	}
}

impl Calculator {
	/// Initialize a [`Calculator`] from a ChemApp dll file and a datafile
	pub fn from_library(libname: &str, datfile: &str)->Result<Calculator, ChemAppError>{
		let engine = Engine::new(libname).unwrap();
		Self::init_engine(&engine, datfile);
		let components : Vec<String> = (0..engine.tqnosc()?).into_iter().map(|idx| engine.tqgnsc(idx+1)).filter_map(|r| r.ok()).collect();
		let transform = Transform::from_formulas_s(&components, &components);
		return Ok(Calculator {
			engine: engine,
			file: datfile.to_string(),
			number_isothermal: 0,
			number_target_t: 0,
			transform: transform,
		});
	}
	
	fn init_engine(engine: &Engine, datfile: &str)->Result<(),ChemAppError>{
		engine.tqini()?;
		
		Self::load_datafile(engine, datfile)?;
		return Ok(());
	}
	
	fn load_datafile(engine: &Engine, datfile: &str)->Result<(),ChemAppError>{
		let res = get_extension_from_filename(datfile);
		match res {
			Some(extension) => {
				//println!("extension = {}", &extension);
				match extension.as_ref() {
					"dat" => {
						// loading ASCII file
						//println!("HERE");
						engine.tqopna(datfile, 10); // unit number
						engine.tqrfil();
						engine.tqclos(10);
					}
					"cst" => {
						// loading Transparent-Header File
						engine.tqopnt(datfile, 10);
						engine.tqrcst();
						engine.tqclos(10);
					}
					"bin" => {
						// loading a Binary File
						engine.tqopnb(datfile, 10);
						engine.tqrbin();
						engine.tqclos(10);
					}
					_ => {
						return Err(ChemAppError::OtherError(format!("{} is not a recognized datafile extension for {}", extension, datfile)));
					}
				}
			}
			None => {
				return Err(ChemAppError::OtherError(format!("{} has no extension", datfile)));
			}
		}
		return Ok(());
	}
	/// set a formula transform for input compositions
	pub fn set_transform<T: AsRef<str>>(&mut self, basis: &[T])->Result<(),ChemAppError>{
		self.transform = Transform::from_formulas_s(&self.names_components()?, basis);
		return Ok(());
	}
	
	pub fn reset(&mut self)->Result<(),ChemAppError>{
		self.engine.tqremc(-2)?;
		return Ok(());
	}
	
	pub fn components(&self)->Result<Range<usize>,ChemAppError>{
		return Ok(1..self.engine.tqnosc()?+1);
	}
	
	pub fn phases(&self)->Result<Range<usize>,ChemAppError>{
		return Ok(1..self.engine.tqnop()?+1);
	}
	
	
	pub fn names_components(&self)->Result<Vec<String>,ChemAppError>{
		return Ok((0..self.engine.tqnosc()?).into_iter().map(|idx| self.engine.tqgnsc(idx+1)).filter_map(|r| r.ok()).collect());
	}
	
	pub fn names_phases(&self)->Result<Vec<String>,ChemAppError>{
		return Ok((0..self.engine.tqnop()?).into_iter().map(|idx| self.engine.tqgnp(idx+1)).filter_map(|r| r.ok()).collect());
	}
	
	fn calculate_isothermal(&mut self, x_i: &DVector<f64>, temp: f64)->Result<(),ChemAppError>{
		self.reset();
		self.engine.tqsetc("T", 0, 0, temp)?;
		for k in 0..x_i.len(){
			self.engine.tqsetc("IA", 0, k+1, x_i[k])?;
		}
		//self.engine.tqshow();
		self.engine.tqce(" ", 0, 0, (10.0, 6000.0))?;
		self.number_isothermal += 1;
		return Ok(());
	}
	/// Perform a no-target isothermal calculation for an input composition and a temperature, use dynamic vectors
	pub fn calculate_isothermal_d(&mut self, compositions: &DVector<f64>, temp: f64)->Result<(),ChemAppError>{
		return self.calculate_isothermal(&self.transform.transform_f2i_d2d(compositions), temp);
	}
	/// Perform a no-target isothermal calculation for an input composition and a temperature, use static vectors
	pub fn calculate_isothermal_s<const N: usize>(&mut self, compositions: &SVector<f64,N>, temp: f64)->Result<(),ChemAppError>{
		//println!("compositions = {:?}, N = {}", &compositions, N);
		//println!("f2i: {:?}", &self.transform.f2i);
		return self.calculate_isothermal(&self.transform.transform_f2i_s2d(compositions), temp);
	}
	
	fn calculate_target_t(&mut self, x_i: &DVector<f64>, target: usize, interval: (f64,f64))->Result<(),ChemAppError>{
		self.reset();
		//let x_i = self.transform.transform_f2i(compositions);
		//let x_i = compositions; // TODO!!!
		for k in 0..x_i.len(){
			self.engine.tqsetc("IA", 0, k+1, x_i[k])?;
		}
		self.engine.tqsetc("A", target, 0, 0.0)?;
		self.set_clim(interval, true);
		//self.engine.tqshow()?;
		self.engine.tqce("T", 0, 0, interval)?;
		self.number_target_t += 1;
		return Ok(());
	}
	/// set (tlower, thigh) limits
	pub fn set_clim(&self, interval: (f64,f64), inverse_order: bool){
		if inverse_order {
		//println!("<THIGH>, TLOW");
		let res = self.engine.tqclim("THIGH", interval.1);
		match res {
			Ok(_) => {
				//println!("THIGH, <TLOW>");
				self.engine.tqclim("TLOW", interval.0).unwrap();
			}
			Err(_) => {
				self.set_clim(interval, false);
			}
		}
	} else {
		//println!("<TLOW>, THIGH");
		let res = self.engine.tqclim("TLOW", interval.0);
		match res {
			Ok(_) => {
				//println!("TLOW, <THIGH>");
				self.engine.tqclim("THIGH", interval.1).unwrap();
			}
			Err(_) => {
				self.set_clim(interval, true);
			}
		}
	}
	}
	/// Perform a T-target calculation for an input composition and a temperature, use dynamic vectors
	pub fn calculate_target_t_d(&mut self, compositions: &DVector<f64>, target: usize, interval: (f64,f64))->Result<(),ChemAppError>{
		return self.calculate_target_t(&self.transform.transform_f2i_d2d(compositions), target, interval);
	}
	/// Perform a T-target calculation for an input composition and a temperature, use static vectors
	pub fn calculate_target_t_s<const N: usize>(&mut self, compositions: &SVector<f64,N>, target: usize, interval: (f64,f64))->Result<(),ChemAppError>{
		return self.calculate_target_t(&self.transform.transform_f2i_s2d(compositions), target, interval);
	}
	
	fn calculate_target_x_from_left(&mut self, x1: &DVector<f64>, x2: &DVector<f64>, temp: f64, target: usize)->Result<(),ChemAppError>{
		let n_iter_max = 10;
		let mut x_initial = x1.clone();
		let mut x_other = x2;
		for k in 0..n_iter_max{
			x_initial = (&x_initial + x_other) * 0.5;
			self.calculate_isothermal(&x_initial, temp)?;
			if self.phases()?.phases_stable(&self).any(|pid| pid == target) {
				if self.phases()?.phases_stable(&self).count() > 1 {
					return Ok(());
				}
				x_other = x2;
			}
			x_other = x1;
		}
		return Err(ChemAppError::OtherError("Cannot converge X target".to_string()));
	}
	/// Perform a composition search starting from `x1` until a required phase is met, use dynamic vectors
	pub fn calculate_target_x_from_left_d(&mut self, x1: &DVector<f64>, x2: &DVector<f64>, temp: f64, target: usize)->Result<(),ChemAppError>{
		return self.calculate_target_x_from_left(&self.transform.transform_f2i_d2d(x1), &self.transform.transform_f2i_d2d(x2), temp, target);
	}
	/// Perform a composition search starting from `x1` until a required phase is met, use static vectors
	pub fn calculate_target_x_from_left_s<const N: usize>(&mut self, x1: &SVector<f64,N>, x2: &SVector<f64,N>, temp: f64, target: usize)->Result<(),ChemAppError>{
		return self.calculate_target_x_from_left(&self.transform.transform_f2i_s2d(x1), &self.transform.transform_f2i_s2d(x2), temp, target);
	}
	
	pub fn system_temperature(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("T", 0, 0);
	}
	
	pub fn system_pressure(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("P", 0, 0);
	}
	
	pub fn phase_H(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("H", indexp, 0);
	}
	
	pub fn phase_G(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("G", indexp, 0);
	}
	
	pub fn phase_S(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("S", indexp, 0);
	}
	
	pub fn phase_CP(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CP", indexp, 0);
	}
	
	pub fn phase_V(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("V", indexp, 0);
	}
	
	pub fn phase_HM(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("HM", indexp, 0);
	}
	
	pub fn phase_GM(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("GM", indexp, 0);
	}
	
	pub fn phase_SM(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("SM", indexp, 0);
	}
	
	pub fn phase_CPM(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CPM", indexp, 0);
	}
	
	pub fn phase_VM(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("VM", indexp, 0);
	}
	/// Composition of a phase (transformed), dynamic vectors
	pub fn phase_composition(&self, indexp: usize)->Result<DVector<f64>, ChemAppError>{
		let ncomp = self.engine.tqnosc()?;
		let mut xp: DVector<f64> = DVector::zeros(ncomp);
		for k in 0..ncomp {
			xp[k] = self.engine.tqgetr("XP", indexp, k+1)?;
		}
		let mut xe : DVector<f64> = self.transform.transform_i2f_d2d(&xp);
		xe /= xe.sum();
		return Ok(xe);
	}
	/// Composition of a phase (transformed), static vectors
	pub fn phase_composition_static<const N: usize>(&self, indexp: usize)->Result<SVector<f64,N>, ChemAppError>{
		let ncomp = self.engine.tqnosc()?;
		let mut xp: DVector<f64> = DVector::zeros(ncomp);
		for k in 0..ncomp {
			xp[k] = self.engine.tqgetr("XP", indexp, k+1)?;
		}
		let mut xe : SVector<f64,N> = self.transform.transform_i2f_d2s(&xp);
		xe /= xe.sum();
		return Ok(xe);
	}
	
}

/********************************************************************************************************/
/********************************************************************************************************/

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
	
	fn components_AC<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("AC", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_MU<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("MU", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_A<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("A", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn components_X<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("X", 0, idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
}


/********************************************************************************************************/
/********************************************************************************************************/

pub trait PhaseIterator where Self : Sized + Iterator<Item=usize>{
	
	fn phases_valid(self, calculator: &Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool>>{
		let nphases = calculator.engine.tqnop().unwrap();
		return self.filter(Box::new(move |idx : &usize| *idx > 0 && *idx <= nphases));
	}
	
	fn phases_stable<'a>(self, calculator: &'a Calculator)->Filter<Self, Box<dyn Fn(&usize)->bool + 'a>>{
		return self.filter(Box::new(move |idx: &usize| calculator.engine.tqgetr("AC", *idx, 0).unwrap() > 0.9999));
	}
	
	fn phases_models<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->String + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqmodl(idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
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
	
	fn phases_compositions<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->DVector<f64> + 'a>>{
		let closure = move |idx: usize| {
			// TODO make sure it works correctly with other units!
			let ncomp = calculator.engine.tqnosc().unwrap();
			let mut xp : DVector<f64> = DVector::zeros(ncomp);
			for k in 0..ncomp {
				xp[k] = calculator.engine.tqgetr("XP", idx, k+1).unwrap();
			}
			let mut xe = calculator.transform.transform_i2f_d2d(&xp);
			xe /= xe.sum();
			return xe;
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_AC<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("AC", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_A<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("A", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_MU<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("MU", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_H<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("H", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_G<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("G", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_S<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("S", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_CP<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CP", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_V<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("V", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_HM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("HM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_GM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("GM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_SM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("SM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_CPM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CPM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_VM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("VM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_names<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->String + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgnp(idx).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
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
/********************************************************************************************************/
/********************************************************************************************************/



/********************************************************************************************************/
/********************************************************************************************************/

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
	
	fn constituents_names<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->(String,String) + 'a>>{
		let closure = move |(indexp, indexc): (usize, usize)| {
			return (calculator.engine.tqgnp(indexp).unwrap(), calculator.engine.tqgnpc(indexp, indexc).unwrap());
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_A<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("A", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_AC<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("AC", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_MU<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("MU", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_H<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("H", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_G<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("G", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_S<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("S", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_CP<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CP", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_V<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("V", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_HM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("HM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_GM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("GM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_SM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("SM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_CPM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CPM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_VM<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("VM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
}

