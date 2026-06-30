// chemapp::calculator.rs

//! A high level submodule for easy operations on ChemApp library - avoid unnecessary boilerplate code. The user is still free to use both `native` and `Calculator` style function in a free manner.
//! An important feature of `Calculator` is the ability of predefining the composition basis - a useful feature, for example, in oxide systems, where system components are defined as elements, but the compositions should be entered as oxides (CaO, FeO, SiO2, etc). 

use std::path::Path;
use std::ffi::OsStr;
//use std::collections::{HashMap};
use std::ops::{Range};
use nalgebra::{DVector, SVector, Vector, Dim, Storage};
use tempfile::NamedTempFile;
use chemformula::{Transform};

use crate::{Engine, error::{ChemAppError}};
use crate::cache::{ParameterCache};
use crate::parse::*;
use crate::iterator::phase::PhaseIterator;

/*****************************************************************************************************************************************************************************************************/
/*****************************************************************************************************************************************************************************************************/

/// A higher-level abtraction entity.
#[derive(Debug)]
pub struct Calculator {
	/// a loaded instance of ChemApp engine
	pub engine: Engine,
	/// a copy of model parameters, allowing to restore delta inputs
	pub cache: Option<ParameterCache>, 
	/// datafile 
	pub file: String, 
	/// a custom file to output errors
	pub nondefault_errunit : Option<(String,usize)>,
	/// isothermal calculation counter
	pub number_isothermal: usize,
	/// target calculation counter
	pub number_target_t: usize,
	 /// instead of raw input using the system components basis, the user can define a custom formula basis; the transform is handled internally
	pub transform: Transform,
}

/// A helper function to tell whether we deal with an open format *.DAT, transparent header *.CST, or binary formats/
fn get_extension_from_filename(filename: &str)->Option<String>{
	return Path::new(filename).extension().and_then(|s| OsStr::to_str(s).and_then(|s| Some(s.to_lowercase())));
}

impl Default for Calculator {
	fn default()->Calculator{
		return Calculator{
			engine: Engine::default(),
			cache: None,
			file: r"".to_string(),
			nondefault_errunit : None,
			number_isothermal: 0,
			number_target_t: 0,
			transform: Transform::default(),
		};
	}
}

impl Calculator {
	/// Initialize a [`Calculator`] from a ChemApp dll file and a datafile
	pub fn from_library(libname: & str, datfile: & str)->Result<Calculator, ChemAppError>{
		let engine = Engine::new(libname).unwrap();
		Self::init_engine(&engine, datfile)?;
		let components : Vec<String> = (0..engine.tqnosc()?).into_iter().map(|idx| engine.tqgnsc(idx+1)).filter_map(|r| r.ok()).collect();
		let transform = Transform::new(&components, &components, true);
		return Ok(Calculator {
			engine: engine,
			cache: None,
			file: datfile.to_string(),
			nondefault_errunit: None,
			number_isothermal: 0,
			number_target_t: 0,
			transform: transform.unwrap(),
		});
	}
	/// Initializes the ChemApp interface and preconfigures it with the thermodynamic info from a datafile.
	pub fn init_engine(engine: &Engine, datfile: &str)->Result<(),ChemAppError>{
		engine.tqini()?;
		Self::load_datafile(engine, datfile)?;
		return Ok(());
	}
	/// Initializes a ChemApp interface without a datafile
	pub fn from_library_unloaded(libname: &str)->Result<Calculator,ChemAppError>{
		let engine = Engine::new(libname).unwrap();
		engine.tqini()?;
		return Ok(Calculator {
			engine: engine,
			cache : None,
			file  : "".to_string(),
			nondefault_errunit : None,
			number_isothermal : 0,
			number_target_t : 0,
			transform : Transform::default(),
		});
	}
	
	/// A higher-level abstraction over datafile handling, this function is only needed to be called once while the datafile type (open or transparent header) is automatically detected.
	pub fn load_datafile(engine: &Engine, datfile: &str)->Result<(),ChemAppError>{
		let res = get_extension_from_filename(datfile);
		match res {
			Some(extension) => {
				//println!("extension = {}", &extension);
				match extension.as_ref() {
					"dat" => {
						// loading ASCII file
						engine.tqopna(datfile, 10)?; // unit number
						engine.tqrfil()?;
						engine.tqclos(10)?;
					}
					"cst" => {
						// loading Transparent-Header File
						engine.tqopnt(datfile, 10)?;
						engine.tqrcst()?;
						engine.tqclos(10)?;
					}
					"bin" => {
						// loading a Binary File
						engine.tqopnb(datfile, 10)?;
						engine.tqrbin()?;
						engine.tqclos(10)?;
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
	/// Set a formula transform for input compositions
	pub fn set_transform<T: AsRef<str>>(&mut self, basis: &[T])->Result<(),ChemAppError>{
		self.transform = Transform::new(&self.names_components().unwrap(), basis, true).unwrap();
		return Ok(());
	}
	/// Internally, creates a temporary file (deleted once the current `Calculator` instance is dropped) to redirect ChemApp outputs; this is a useful feature in environments where console window is not available.
	pub fn redirect_error_to_temp(&mut self)->Result<(),ChemAppError>{
		let parent = Path::new(&self.engine.library_name).parent().expect("Does not have a parent directory").to_path_buf();
		let temp_file = NamedTempFile::new_in(parent).unwrap();
		let temp_file_ = temp_file.keep().unwrap().1;
		let filename :String = temp_file_.to_string_lossy().into_owned();
		let unit = 30;
		self.engine.tqopen(&filename,unit)?;
		self.engine.tqcio("ERROR",unit)?;
		self.nondefault_errunit = Some((filename,unit));
		return Ok(());
	}
	/// Resets all input conditions to prepare for another calculation (with the same datafile).
	pub fn reset(&self)->Result<(),ChemAppError>{
		self.engine.tqremc(-2)?;
		return Ok(());
	}
	/// Iterates over system component indices.
	pub fn components(&self)->Result<Range<usize>,ChemAppError>{
		return Ok(1..self.engine.tqnosc()?+1);
	}
	/// Iterates over phase indices.
	pub fn phases(&self)->Result<Range<usize>,ChemAppError>{
		return Ok(1..self.engine.tqnop()?+1);
	}
	/// Iterates over system component names.
	pub fn names_components(&self)->Result<Vec<String>,ChemAppError>{
		return Ok((0..self.engine.tqnosc()?).into_iter().map(|idx| self.engine.tqgnsc(idx+1)).filter_map(|r| r.ok()).collect());
	}
	/// Iterates over phase names
	pub fn names_phases(&self)->Result<Vec<String>,ChemAppError>{
		return Ok((0..self.engine.tqnop()?).into_iter().map(|idx| self.engine.tqgnp(idx+1)).filter_map(|r| r.ok()).collect());
	}
	/// A simple isothermal calculation (temperature + initial composition in the pre-transformed basis).
	fn calculate_isothermal_(&self, x_i: &DVector<f64>, temp: f64)->Result<(),ChemAppError>{
		self.reset()?;
		self.engine.tqsetc("T", 0, 0, temp)?;
		for k in 0..x_i.len(){
			self.engine.tqsetc("IA", 0, k+1, x_i[k])?;
		}
		//self.engine.tqshow();
		self.engine.tqce(" ", 0, 0, (10.0, 6000.0))?;
		//self.number_isothermal += 1;
		return Ok(());
	}
	/// Perform a no-target isothermal calculation for an input composition and a temperature, use dynamic vectors; TODO check the composition transformations
	pub fn calculate_isothermal<D: Dim, S: Storage<f64,D>>(& self, compositions: &Vector<f64,D,S>, temp: f64)->Result<(),ChemAppError>{
		return self.calculate_isothermal_(&self.transform.transform_final2init(compositions, false, false, false).column(0).into_owned(), temp);
	}
	
	fn calculate_target_t_(&self, x_i: &DVector<f64>, masterphase: usize, target: usize, interval: (f64,f64), precipitation: bool, fixed: Option<usize>, adjusting: Option<usize>)->Result<(),ChemAppError>{
		// set non-compositional conditions
		let nitermax = 10usize;
		let val = if precipitation {-0.5} else {0.0};
		self.engine.tqsetc("A", target, 0, val)?;
		self.set_clim(interval, true);
		// set compositions
		let mut xvar : DVector<f64> = x_i.clone();
		//let mut xvarprev : DVector<f64> = xvar.clone();
		let mut xvarprev : DVector<f64>;
		match (fixed,adjusting) {
			(Some(sidxf),Some(sidxa)) => {
				for iter in 0..nitermax {
					for k in 0..xvar.len(){self.engine.tqsetc("IA", 0, k+1, xvar[k])?;}
					//self.engine.tqshow()?;
					self.engine.tqce("T", 0, 0, interval)?;
					xvarprev = xvar.clone();
					let xfold = xvar[sidxf-1];
					let xaold = xvar[sidxa-1];
					let xfnew = self.engine.tqgetr("XP", masterphase, sidxf)?;
					let xanew = self.engine.tqgetr("XP", masterphase, sidxa)?;
					let tliq  = self.engine.tqgetr("T", 0, 0)?;
					xvar[sidxa-1] = xvar[sidxf-1]*xanew/xfnew;
					if iter > 0 {
						xvar = (&xvar + &xvarprev)*0.5;
					}
					println!("iter = {:?}, tliq = {:?}, xfold = {:?}, xaold = {:?}, xfnew = {:?}, xanew = {:?}, xvarprev = {:?}, xvar = {:?}", &iter, &tliq, &xfold, &xaold, &xfnew, &xanew, &xvarprev, &xvar);
					if (&xvar-&xvarprev).abs().sum() < 5e-3 {return Ok(());}
				}
			}
			(None,None) => {
				for k in 0..xvar.len(){self.engine.tqsetc("IA", 0, k+1, xvar[k])?;}
				// perform calculation
				//self.engine.tqshow()?;
				self.engine.tqce("T", 0, 0, interval)?;
			}
			_ => {panic!("Both or none of fixed and adjusting system components must be defined");}
		}
		//self.number_target_t += 1;
		return Ok(());
	}
	/// set (tlower, thigh) limits
	pub fn set_clim(&self, interval: (f64,f64), inverse_order: bool){
		if inverse_order {
		//println!("<THIGH>, TLOW");
		let res = self.engine.tqclim("THIGH", interval.1);
		match res {
			Ok(_) => {
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
	/// Perform a T-target calculation for an input composition and a temperature, use dynamic vectors; TODO check the composition transformations
	pub fn calculate_target_t<D: Dim, S: Storage<f64,D>>(&self, compositions: &Vector<f64,D,S>, masterphase: usize, target: usize, interval: (f64,f64), precipitation: bool, fixed: Option<usize>, adjusting: Option<usize>)->Result<(),ChemAppError>{
		return self.calculate_target_t_(&self.transform.transform_final2init(compositions, false, false, false).column(0).into_owned(), masterphase, target, interval, precipitation, fixed, adjusting);
	}
	
	fn calculate_target_x_from_left_(&self, x1: &DVector<f64>, x2: &DVector<f64>, temp: f64, target: usize)->Result<(),ChemAppError>{
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
				//x_other = x2;
			}
			x_other = x1;
		}
		return Err(ChemAppError::OtherError("Cannot converge X target".to_string()));
	}
	/// Perform a composition search starting from `x1` until a required phase is met, use dynamic vectors; TODO check the composition transformations
	pub fn calculate_target_x_from_left<D: Dim, S: Storage<f64,D>>(&self, x1: &Vector<f64,D,S>, x2: &Vector<f64,D,S>, temp: f64, target: usize)->Result<(),ChemAppError>{
		return self.calculate_target_x_from_left_(&self.transform.transform_final2init(x1, false, false, false).column(0).into_owned(), &self.transform.transform_final2init(x2, false, false, false).column(0).into_owned(), temp, target);
	}
	
	/// Returns the resulting system temperature
	pub fn system_temperature(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("T", 0, 0);
	}
	/// Returns the resulting system pressure
	pub fn system_pressure(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("P", 0, 0);
	}
	/// Returns the enthalpy of a phase
	pub fn phase_h(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("H", indexp, 0);
	}
	/// Returns the Gibbs free energy of a phase
	pub fn phase_g(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("G", indexp, 0);
	}
	/// Returns the entropy of a phase
	pub fn phase_s(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("S", indexp, 0);
	}
	/// Returns the heat capacity of a phase
	pub fn phase_cp(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CP", indexp, 0);
	}
	/// Return the volume of a phase
	pub fn phase_v(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("V", indexp, 0);
	}
	/// Returns the enthalpy of a phase per amount unit
	pub fn phase_hm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("HM", indexp, 0);
	}
	/// Returns the Gibbs free energy of a phase per amount unit
	pub fn phase_gm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("GM", indexp, 0);
	}
	/// Returns the entropy of a phase per amount unit
	pub fn phase_sm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("SM", indexp, 0);
	}
	/// Returns the heat capacity of a phase per amount unit
	pub fn phase_cpm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CPM", indexp, 0);
	}
	/// Returns the volume of a phase per amount unit
	pub fn phase_vm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("VM", indexp, 0);
	}
	/// Composition of a phase (transformed), dynamic vectors
	pub fn phase_composition(&self, indexp: usize)->Result<DVector<f64>, ChemAppError>{
		let ncomp = self.engine.tqnosc()?;
		let mut xp: DVector<f64> = DVector::zeros(ncomp);
		for k in 0..ncomp {
			xp[k] = self.engine.tqgetr("XP", indexp, k+1)?;
		}
		let xe : DVector<f64> = self.transform.transform_init2final(&xp, false, false, true).column(0).into_owned();
		return Ok(xe);
	}
	/// Returns the stoichiometry of a phase constituent in the input basis
	pub fn constituent_stoichiometry_full(&self, indexp: usize, indexc: usize)->Result<DVector<f64>, ChemAppError>{
		let comp_s : DVector<f64> = self.engine.tqstpc(indexp, indexc).unwrap().0.into();
		let comp_ : DVector<f64> = self.transform.transform_init2final(&comp_s, false, false, false).column(0).into_owned();
		return Ok(comp_);
	}
	/// Number of independent formula units in the input basis
	pub fn number_endmembers(&self)->usize{
		return self.transform.number_final();
	}
	/// Returns the normalized stoichiometry of a phase constituent in the input basis
	pub fn constituent_stoichiometry_reduced(&self, indexp: usize, indexc: usize)->Result<(DVector<f64>,f64),ChemAppError>{
		let comp_s : DVector<f64> = self.engine.tqstpc(indexp, indexc).unwrap().0.into();
		let mut comp_ : DVector<f64> = self.transform.transform_init2final(&comp_s, false, false, false).column(0).into_owned();
		let ntotal = comp_.sum();
		comp_ /= ntotal;
		return Ok((comp_,ntotal));
	}
	/// Lists the Gibbs free energy excess interactions in a phase as-is (species indices are used which are subject to change from a datafile to a datafile)
	pub fn interactions_ge_expanded(&self, indexp: usize)->Result<Vec<String>,ChemAppError>{
		let interactions0 : Vec<String> = self.engine.tqlpar(indexp, "G")?;
		let mut interactions : Vec<String> = Vec::with_capacity(interactions0.len());
		for k in 0..interactions0.len(){
			match convert_ge_interaction(&self.engine,indexp,&interactions0[k]){
				Ok(s) => {
					interactions.push(s.1);
				}
				Err(_) => {continue;}
			}
		}
		return Ok(interactions);
	}
	
	/// Lists the magnetic interactions in a phase as-is (species indices are used which are subject to change from a datafile to a datafile)
	pub fn interactions_magn_expanded(&self, indexp: usize)->Result<Vec<String>,ChemAppError>{
		let interactions0 : Vec<String> = self.engine.tqlpar(indexp, "M")?;
		let mut interactions : Vec<String> = Vec::with_capacity(interactions0.len());
		for k in 0..interactions0.len(){
			match convert_magn_interaction(&self.engine,indexp,&interactions0[k]){
				Ok(s) => {
					interactions.push(s.1);
				}
				Err(_) => {continue;}
			}
		}
		return Ok(interactions);
	}
	
	/// Lists post-processed Gibbs free energy excess interactions in a phase (species indices are replaced by species names).
	pub fn interactions_ge_expanded_species(&self,indexp: usize)->Result<Vec<Vec<String>>,ChemAppError>{
		let interactions0 : Vec<String> = self.engine.tqlpar(indexp, "G")?;
		let mut interactions : Vec<Vec<String>> = Vec::with_capacity(interactions0.len());
		for k in 0..interactions0.len(){
			match convert_ge_interaction_species(&self.engine,indexp,&interactions0[k]){
				Ok(s) => {
					interactions.push(s.1);
				}
				Err(_) => {continue;}
			}
		}
		return Ok(interactions);
	}
	/// Lists post-processed magnetic excess interactions in a phase (species indices are replaced by species names).
	pub fn interactions_magn_expanded_species(&self, indexp: usize)->Result<Vec<Vec<String>>, ChemAppError>{
		let interactions0 : Vec<String> = self.engine.tqlpar(indexp, "M")?;
		let mut interactions : Vec<Vec<String>> = Vec::with_capacity(interactions0.len());
		for k in 0..interactions0.len(){
			match convert_magn_interaction_species(&self.engine,indexp,&interactions0[k]){
				Ok(s) => {
					interactions.push(s.1);
				}
				Err(_) => {continue;}
			}
		}
		return Ok(interactions);
	}
	
}

/// Custom `Drop` re-implementation to ensure any temporary files are deleted.
impl Drop for Calculator {
	
	fn drop(&mut self){
		match &self.nondefault_errunit {
			Some((filename,unit)) => {
				let _ = self.engine.tqclos(*unit);
				let _ = std::fs::remove_file(&filename);
			}
			None => {}
		}
	}
	
}