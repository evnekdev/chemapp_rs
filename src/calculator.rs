// chemapp::calculator.rs

/// A high level module for easy operations on ChemApp library

use std::cmp::{Ordering};
use std::path::Path;
use std::ffi::OsStr;
use std::collections::{HashMap};
use std::iter::{Filter, Map, FlatMap};
use std::ops::{Range};
use nalgebra::{DVector, SVector, Matrix, Dim, DimName, U1, Storage};
use nom::{IResult,sequence::{tuple,delimited},
branch::{alt},
character::complete::{u32,char,multispace1,multispace0,digit1},
combinator::{map_res},
bytes::complete::{tag,tag_no_case},
multi::{separated_list1}
};
use tempfile::NamedTempFile;

use crate::{Engine, error::{ChemAppError}};
use crate::interactions::{ParameterCache};
use chemformula::{Transform};


/********************************************************************************************************/
/********************************************************************************************************/

#[derive(Debug)]
pub struct Calculator {
	pub engine: Engine,
	pub cache: Option<ParameterCache>,
	pub file: String,
	pub nondefault_errunit : Option<(String,usize)>,
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
		//println!("Calculator::from_library");
		let engine = Engine::new(libname).unwrap();
		Self::init_engine(&engine, datfile)?;
		//println!("Self::init_engine");
		let components : Vec<String> = (0..engine.tqnosc()?).into_iter().map(|idx| engine.tqgnsc(idx+1)).filter_map(|r| r.ok()).collect();
		let transform = Transform::from_formulas_s(&components, &components);
		return Ok(Calculator {
			engine: engine,
			cache: None,
			file: datfile.to_string(),
			nondefault_errunit: None,
			number_isothermal: 0,
			number_target_t: 0,
			transform: transform,
		});
	}
	
	fn init_engine(engine: &Engine, datfile: &str)->Result<(),ChemAppError>{
		engine.tqini()?;
		//println!("engine.tqini()?");
		Self::load_datafile(engine, datfile)?;
		//println!("Self::load_datafile");
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
	/// set a formula transform for input compositions
	pub fn set_transform<T: AsRef<str>>(&mut self, basis: &[T])->Result<(),ChemAppError>{
		self.transform = Transform::from_formulas_s(&self.names_components()?, basis);
		//println!("components = {:?}", &self.names_components()?);
		//println!("self.transform.f2i = {}", &self.transform.f2i);
		//println!("self.transform.i2f = {}", &self.transform.i2f);
		return Ok(());
	}
	/*
	pub fn redirect_error_to_temp(&mut self)->Result<(),ChemAppError>{
		let mut path = Path::new(&self.engine.library_name).to_path_buf();
		path.set_extension("tmp");
		let filename = path.to_string_lossy().into_owned();
		let unit = 30;
		self.engine.tqopen(&filename,unit)?;
		self.engine.tqcio("ERROR",unit)?;
		self.nondefault_errunit = Some((filename,unit));
		return Ok(());
	}
	*/
	pub fn redirect_error_to_temp(&mut self)->Result<(),ChemAppError>{
		let mut parent = Path::new(&self.engine.library_name).parent().expect("Does not have a parent directory").to_path_buf();
		let temp_file = NamedTempFile::new_in(parent).unwrap();
		//let filename = temp_file.path().to_string_lossy().into_owned();
		let temp_file_ = temp_file.keep().unwrap().1;
		let filename :String = temp_file_.to_string_lossy().into_owned();
		let unit = 30;
		self.engine.tqopen(&filename,unit)?;
		self.engine.tqcio("ERROR",unit)?;
		self.nondefault_errunit = Some((filename,unit));
		return Ok(());
	}
	
	pub fn reset(&self)->Result<(),ChemAppError>{
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
	
	fn calculate_isothermal(&self, x_i: &DVector<f64>, temp: f64)->Result<(),ChemAppError>{
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
	/// Perform a no-target isothermal calculation for an input composition and a temperature, use dynamic vectors
	pub fn calculate_isothermal_d(& self, compositions: &DVector<f64>, temp: f64)->Result<(),ChemAppError>{
		return self.calculate_isothermal(&self.transform.transform_f2i_d2d(compositions), temp);
	}
	/// Perform a no-target isothermal calculation for an input composition and a temperature, use static vectors
	pub fn calculate_isothermal_s<const N: usize>(&self, compositions: &SVector<f64,N>, temp: f64)->Result<(),ChemAppError>{
		//println!("compositions = {:?}, N = {}", &compositions, N);
		//println!("f2i: {:?}", &self.transform.f2i);
		return self.calculate_isothermal(&self.transform.transform_f2i_s2d(compositions), temp);
	}
	
	fn calculate_target_t(&self, x_i: &DVector<f64>, masterphase: usize, target: usize, interval: (f64,f64), precipitation: bool, fixed: Option<usize>, adjusting: Option<usize>)->Result<(),ChemAppError>{
		// set non-compositional conditions
		let nitermax = 10usize;
		let val = if precipitation {-0.5} else {0.0};
		self.engine.tqsetc("A", target, 0, val)?;
		self.set_clim(interval, true);
		// set compositions
		let mut xvar : DVector<f64> = x_i.clone();
		let mut xvarprev : DVector<f64> = xvar.clone();
		match (fixed,adjusting) {
			(Some(sidxf),Some(sidxa)) => {
				for iter in 0..nitermax {
					for k in 0..xvar.len(){self.engine.tqsetc("IA", 0, k+1, xvar[k])?;}
					//self.engine.tqshow()?;
					self.engine.tqce("T", 0, 0, interval)?;
					xvarprev = xvar.clone();
					/*
					for k in 0..xvar.len(){xvar[k] = self.engine.tqgetr("A", 0, k+1)?;}
					let xf = xvar[sidxf-1];
					let xa = xvar[sidxa-1];
					*/
					let xfold = xvar[sidxf-1];
					let xaold = xvar[sidxa-1];
					let xfnew = self.engine.tqgetr("XP", masterphase, sidxf)?;
					let xanew = self.engine.tqgetr("XP", masterphase, sidxa)?;
					/*
					let xfnew = self.engine.tqgetr("A", 0, sidxf)?;
					let xanew = self.engine.tqgetr("A", 0, sidxa)?;
					*/
					let tliq  = self.engine.tqgetr("T", 0, 0)?;
					//println!("xfold = {:?}, xaold = {:?}, xfnew = {:?}, xanew = {:?}", &xfold, &xaold, &xfnew, &xanew);
					//xvar[sidxf-1] = xvarprev[sidxf-1];
					//xvar[sidxa-1] = xaold / xfold / xfnew;
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
	/// Perform a T-target calculation for an input composition and a temperature, use dynamic vectors
	pub fn calculate_target_t_d(&self, compositions: &DVector<f64>, masterphase: usize, target: usize, interval: (f64,f64), precipitation: bool, fixed: Option<usize>, adjusting: Option<usize>)->Result<(),ChemAppError>{
		return self.calculate_target_t(&self.transform.transform_f2i_d2d(compositions), masterphase, target, interval, precipitation, fixed, adjusting);
	}
	/// Perform a T-target calculation for an input composition and a temperature, use static vectors
	pub fn calculate_target_t_s<const N: usize>(&self, compositions: &SVector<f64,N>, masterphase: usize, target: usize, interval: (f64,f64), precipitation: bool, fixed: Option<usize>, adjusting: Option<usize>)->Result<(),ChemAppError>{
		return self.calculate_target_t(&self.transform.transform_f2i_s2d(compositions), masterphase, target, interval, precipitation, fixed, adjusting);
	}
	
	fn calculate_target_x_from_left(&self, x1: &DVector<f64>, x2: &DVector<f64>, temp: f64, target: usize)->Result<(),ChemAppError>{
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
	pub fn calculate_target_x_from_left_d(&self, x1: &DVector<f64>, x2: &DVector<f64>, temp: f64, target: usize)->Result<(),ChemAppError>{
		return self.calculate_target_x_from_left(&self.transform.transform_f2i_d2d(x1), &self.transform.transform_f2i_d2d(x2), temp, target);
	}
	/// Perform a composition search starting from `x1` until a required phase is met, use static vectors
	pub fn calculate_target_x_from_left_s<const N: usize>(&self, x1: &SVector<f64,N>, x2: &SVector<f64,N>, temp: f64, target: usize)->Result<(),ChemAppError>{
		return self.calculate_target_x_from_left(&self.transform.transform_f2i_s2d(x1), &self.transform.transform_f2i_s2d(x2), temp, target);
	}
	
	pub fn system_temperature(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("T", 0, 0);
	}
	
	pub fn system_pressure(&self)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("P", 0, 0);
	}
	
	pub fn phase_h(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("H", indexp, 0);
	}
	
	pub fn phase_g(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("G", indexp, 0);
	}
	
	pub fn phase_s(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("S", indexp, 0);
	}
	
	pub fn phase_cp(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CP", indexp, 0);
	}
	
	pub fn phase_v(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("V", indexp, 0);
	}
	
	pub fn phase_hm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("HM", indexp, 0);
	}
	
	pub fn phase_gm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("GM", indexp, 0);
	}
	
	pub fn phase_sm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("SM", indexp, 0);
	}
	
	pub fn phase_cpm(&self, indexp: usize)->Result<f64,ChemAppError>{
		return self.engine.tqgetr("CPM", indexp, 0);
	}
	
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
	
	pub fn constituent_stoichiometry_full(&self, indexp: usize, indexc: usize)->Result<DVector<f64>, ChemAppError>{
		let comp_s : DVector<f64> = self.engine.tqstpc(indexp, indexc).unwrap().0.into();
		let comp_ : DVector<f64> = self.transform.transform_i2f_d2d(&comp_s);
		return Ok(comp_);
	}
	
	pub fn number_endmembers(&self)->usize{
		return self.transform.number_f();
	}
	
	pub fn constituent_stoichiometry_reduced(&self, indexp: usize, indexc: usize)->Result<(DVector<f64>,f64),ChemAppError>{
		let comp_s : DVector<f64> = self.engine.tqstpc(indexp, indexc).unwrap().0.into();
		let mut comp_ : DVector<f64> = self.transform.transform_i2f_d2d(&comp_s);
		let ntotal = comp_.sum();
		comp_ /= ntotal;
		return Ok((comp_,ntotal));
	}
	
	pub fn constituent_stoichiometry_reduced_static<const N: usize>(&self, indexp: usize, indexc: usize)->Result<(SVector<f64,N>,f64),ChemAppError>{
		let comp_s : DVector<f64> = self.engine.tqstpc(indexp, indexc).unwrap().0.into();
		let mut comp_ : SVector<f64,N> = self.transform.transform_i2f_d2s(&comp_s);
		let ntotal = comp_.sum();
		comp_ /= ntotal;
		return Ok((comp_,ntotal));
	}
	
	pub fn interactions_ge_expanded(&self, indexp: usize)->Result<Vec<String>,ChemAppError>{
		//let interactions : Vec<String> = self.engine.tqlpar(indexp, "G")?.into_iter().map(|s| convert_ge_interaction(&self.engine,indexp,&s).unwrap().1).collect();
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
	
	pub fn interactions_ge_expanded_species(&self,indexp:usize)->Result<Vec<Vec<String>>,ChemAppError>{
		//let interactions : Vec<Vec<String>> = self.engine.tqlpar(indexp,"G")?.into_iter().map(|s| convert_ge_interaction_species(&self.engine,indexp,&s).unwrap().1).collect();
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
		//println!("{:?}", &interactions);
		return Ok(interactions);
	}
	
}
/********************************************************************************************************/
/********************************************************************************************************/

fn parse_speciespower<'a>(s: &'a str)->IResult<&'a str,(usize,&str)>{
	//let (s, (_,species,_,power,_)) = tuple((tag("("),map_res(digit1, str::parse::<u32>),tag(")^["),map_res(digit1, str::parse::<u32>),tag("]")))(s)?;
	let (s, (_,species,_,power,_)) = tuple((tag("("),map_res(digit1, str::parse::<u32>),tag(")^["),alt((digit1,tag("*"))),tag("]")))(s)?;
	return Ok((s, (species as usize, power)));
}

fn parse_interaction_index<'a>(s: &'a str)->IResult<&'a str,(usize,usize)>{
	let (s, (index, _, _, _, nterms, _)) = tuple((map_res(digit1, str::parse::<u32>), tag(":"), multispace1, tag("*"), map_res(digit1, str::parse::<u32>), multispace0))(s)?;
	return Ok((s, (index as usize, nterms as usize)));
}

fn parse_reciprocal_index<'a>(s: &'a str)->IResult<&'a str, usize> {
	let (s, (index,_,_,_,_,_)) = tuple((map_res(digit1, str::parse::<u32>), tag(":"), multispace1, tag("*"), tag("R"),  multispace0))(s)?;
	return Ok((s, index as usize));
}

fn parse_ending_species<'a>(s: &'a str)->IResult<&'a str, usize>{
	let (s, index) = delimited(char('('), map_res(digit1, str::parse::<u32>), char(')'))(s)?;
	return Ok((s, index as usize));
}

fn parse_interaction_type(s: &str) -> IResult<&str, &str> {
	let (s, itype) = delimited(tag("("),alt((tag("Guts"),tag("Quasichemical"),tag("Bragg-Williams-Hillert"),tag("Bragg-Williams"),tag("Reciprocal"),)),tag(")"))(s)?;
	return Ok((s,itype));
}

pub fn convert_ge_interaction_species<'a>(engine: &'a Engine, indexp: usize, s: &'a str)->IResult<&'a str,Vec<String>>{
	//let (s, ((index,nterms), vecc, _, _, _, species0, _, itype)) = tuple(( parse_interaction_index, separated_list1(char('-'), parse_speciespower), multispace1, tag(":"), multispace1, parse_ending_species, multispace1, parse_interaction_type  ))(s)?;
	match tuple(( parse_interaction_index, separated_list1(char('-'), parse_speciespower), multispace1, tag(":"), multispace1, parse_ending_species, multispace1, parse_interaction_type  ))(s) {
		Ok((s, ((index,nterms), vecc, _, _, _, species0, _, itype))) => {
			let nspecies1 = engine.tqnolc(indexp, 1).unwrap();
			//let mut vecc_ : Vec<(String,usize)> = Vec::new();
			let mut vecc_ : Vec<String> = Vec::new();
			for k in 0..vecc.len(){
				let name = if vecc[k].0 <= nspecies1 {
				engine.tqgnlc(indexp, 1, vecc[k].0).unwrap()
				} else {
					engine.tqgnlc(indexp, 2, vecc[k].0+nspecies1).unwrap()
				};
				vecc_.push(name);
			}
			let name0 = if species0 <= nspecies1 {
				engine.tqgnlc(indexp, 1, species0).unwrap()
			} else {
				engine.tqgnlc(indexp, 2, species0-nspecies1).unwrap()
			};
			vecc_.push(name0);
			return Ok((s,vecc_));
		}
		Err(_) => {}
	}
	let (s, (index,species1,species2,species3,species4,itype)) = parse_interaction_reciprocal(s)?;
	let mut vecc_ : Vec<String> = vec![species_name(engine,indexp,species1),species_name(engine,indexp,species2),species_name(engine,indexp,species3),species_name(engine,indexp,species4)];
	return Ok((s,vecc_));
}

fn parse_interaction<'a>(s: &'a str)->IResult<&'a str, ((usize,usize),Vec<(usize,&'a str)>,&'a str,&'a str,&'a str,usize,&'a str,&'a str)> {
	return tuple(( parse_interaction_index, separated_list1(char('-'), parse_speciespower), multispace1, tag(":"), multispace1, parse_ending_species, multispace1, parse_interaction_type  ))(s);
}

fn parse_interaction_reciprocal<'a>(s: &'a str)->IResult<&'a str, (usize,usize,usize,usize,usize,&'a str)> {
	// 504: *R (5)-(7) : (20)-(21) (Reciprocal)
	let (s, (index,species1,_,species2,_,_,_,species3,_,species4,itype)) = tuple(( parse_reciprocal_index, parse_ending_species, tag("-"), parse_ending_species, multispace1, tag(":"), multispace1, parse_ending_species, tag("-"), parse_ending_species, parse_interaction_type))(s)?;
	return Ok((s,(index,species1,species2,species3,species4,itype)));
}

fn species_name(engine: &Engine, indexp: usize, sindex: usize)->String {
	let nspecies1 = engine.tqnolc(indexp, 1).unwrap();
	if sindex < nspecies1 {
		return engine.tqgnlc(indexp, 1, sindex).unwrap();
	}
	return engine.tqgnlc(indexp, 2, sindex-nspecies1).unwrap();
}

pub fn convert_ge_interaction<'a>(engine: &'a Engine, indexp: usize, s: &'a str)->IResult<&'a str,String> {
	//println!("s = {:?}", &s);
	//let (s, ((index,nterms), vecc, _, _, _, species0, _, itype)) = tuple(( parse_interaction_index, separated_list1(char('-'), parse_speciespower), multispace1, tag(":"), multispace1, parse_ending_species, multispace1, parse_interaction_type  ))(s)?;
	//let (s, ((index,nterms), vecc, _, _, _, species0, _, itype)) = parse_interaction(s)?;
	match parse_interaction(s) {
		Ok((s, ((index,nterms), vecc, _, _, _, species0, _, itype))) => {
			//println!("Ok, parse_interaction");
			// convert the indices into species names
			assert!(vecc.len() == 2 || vecc.len() == 3);
			let nspecies1 = engine.tqnolc(indexp, 1).unwrap();
			//let mut vecc_ : Vec<(String,usize)> = Vec::new();
			let mut vecc_ : Vec<(String,&str)> = Vec::new();
			for k in 0..vecc.len(){
				let name = if vecc[k].0 <= nspecies1 {
				engine.tqgnlc(indexp, 1, vecc[k].0).unwrap()
				} else {
				engine.tqgnlc(indexp, 2, vecc[k].0+nspecies1).unwrap()
				};
				vecc_.push((name, vecc[k].1));
			}
			let name0 = if species0 <= nspecies1 {
			engine.tqgnlc(indexp, 1, species0).unwrap()
			} else {
				engine.tqgnlc(indexp, 2, species0-nspecies1).unwrap()
			};
			// assemble the reconstructed interaction parameter
			let interaction = if vecc_.len() == 2 {
				format!("({})^[{}]-({})^[{}]: ({}) ({})", &vecc_[0].0, &vecc_[0].1, &vecc_[1].0, &vecc_[1].1, &name0, &itype)
			} else {
				format!("({})^[{}]-({})^[{}]-({})^[{}]: ({}) ({})", &vecc_[0].0, &vecc_[0].1, &vecc_[1].0, &vecc_[1].1, &vecc_[2].0, &vecc_[2].1, &name0, &itype)
			};
			return Ok((s,interaction));
		}
		Err(_) => {
			println!("Err, parse_interaction");
		}
	}
	let (s, (index,species1,species2,species3,species4,itype)) = parse_interaction_reciprocal(s)?;
	let interaction = format!("({})-({}) : ({})-({}) ({})", &species_name(engine,indexp,species1), &species_name(engine,indexp,species2), &species_name(engine,indexp,species3), &species_name(engine,indexp,species4), &itype);
	return Ok((s,interaction));
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
	
	fn phases_ac<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("AC", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_a<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("A", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_mu<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("MU", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_h<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("H", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_g<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("G", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_s<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("S", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_cp<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CP", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_v<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("V", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_hm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("HM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_gm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("GM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_sm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("SM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_cpm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
		let closure = move |idx: usize| {
			return calculator.engine.tqgetr("CPM", idx, 0).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn phases_vm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut(usize)->f64 + 'a>>{
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
	
	fn constituents_a<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("A", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_ac<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("AC", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_mu<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("MU", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_h<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("H", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_g<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("G", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_s<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("S", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_cp<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CP", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_v<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("V", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_hm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("HM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_gm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("GM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_sm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("SM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_cpm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("CPM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
	fn constituents_vm<'a>(self, calculator: &'a Calculator)->Map<Self, Box<dyn FnMut((usize,usize))->f64 + 'a>>{
		let closure = move |(indexp, indexc): (usize,usize)| {
			return calculator.engine.tqgetr("VM", indexp, indexc).unwrap();
		};
		return self.map(Box::new(closure));
	}
	
}

impl Drop for Calculator {
	
	fn drop(&mut self){
		match &self.nondefault_errunit {
			Some((filename,unit)) => {
				self.engine.tqclos(*unit);
				std::fs::remove_file(&filename);
			}
			None => {}
		}
	}
	
}