// chemapp_rs::interactions.rs

//! A newer submodule which facilitates working with *model parameters* (only available for unencrypted *.dat datafiles). Involves 'tqgdat', 'tqcdat', 'tqgpar' ChemApp routines.
//! Disclaimer - this submodule can be subject to change.

use std::collections::{HashMap};

use super::calculator::{Calculator};
use crate::error::{ChemAppError};
use crate::{Engine};


/*******************************************************************************************************************************************************************************************************************************/

/// An excess free energy interaction in a mixture phase.
#[derive(Debug)]
pub struct InteractionGEMQM {
	indexp: usize,
	index: usize,
	phasename: String,
	text: String,
	values: Vec<f64>,
}

impl InteractionGEMQM {
	
	pub fn reset(&self, engine: &Engine)->Result<(),ChemAppError>{
		for k in 0..6 {
			engine.tqcdat(13,self.index,1,k+1,self.indexp,self.values[k])?;
		}
		return Ok(());
	}
	
}

/*******************************************************************************************************************************************************************************************************************************/
/// An excess magnetic interaction in a mixture phase.
#[allow(dead_code)]
#[derive(Debug)]
pub struct InteractionMagnMQM {
	indexp: usize,
	index: usize,
	phasename: String,
	text: String,
	values: Vec<f64>,
}

impl InteractionMagnMQM {
	
	pub fn reset(&self, engine: &Engine)->Result<(),ChemAppError>{
		//todo!();
		return Ok(());
	}
	
}

/*******************************************************************************************************************************************************************************************************************************/
/// A representation of a mixture phase endmember. Currently, only H298 and S298 are considered.
#[derive(Debug)]
pub struct Endmember {
	indexp: usize,
	indexc: usize,
	phasename: String,
	name: String,
	h298: f64,
	s298: f64,
}

impl Endmember {
	
	pub fn reset(&self, engine: &Engine)->Result<(),ChemAppError>{
		engine.tqcdat(1,0,0,self.indexc,self.indexp, self.h298)?; // H298
		engine.tqcdat(1,0,1,self.indexc,self.indexp, self.s298)?; // S298
		return Ok(());
	}
	
}

/*******************************************************************************************************************************************************************************************************************************/
/// A representation of a `PURE` phase (a stoichiometric compound). Currenly, only H298 and S298 are considered.
#[derive(Debug)]
pub struct Compound {
	indexp: usize,
	phasename: String,
	h298: f64,
	s298: f64,
}

impl Compound {
	
	pub fn reset(&self, engine: &Engine)->Result<(),ChemAppError>{
		engine.tqcdat(1,0,0,1,self.indexp, self.h298)?; // H298
		engine.tqcdat(1,0,1,1,self.indexp, self.s298)?; // S298
		return Ok(());
	}
	
}

/*******************************************************************************************************************************************************************************************************************************/
/// A cache which allows to apply and reset parameter deltas, which is important for construction of sensitivity matrices (delta calc values vs delta parameters).
#[allow(dead_code)]
#[derive(Debug)]
pub struct ParameterCache {
	lookup_ge: HashMap<(String,String),usize>,
	lookup_magn: HashMap<(String,String),usize>,
	lookup_endm: HashMap<(String,String),usize>,
	lookup_cmp: HashMap<String,usize>,
	interactions_ge : Vec<InteractionGEMQM>,
	interactions_magn: Vec<InteractionMagnMQM>,
	endmembers : Vec<Endmember>,
	compounds : Vec<Compound>,
}

impl ParameterCache {
	
	pub fn load_compound(calculator: &Calculator, phasename: &str)->Result<Compound,ChemAppError> {
		let indexp = calculator.engine.tqinp(phasename)?;
		let h298 = calculator.engine.tqgdat(indexp, 1, "H", 0)?;
		let s298 = calculator.engine.tqgdat(indexp, 1, "S", 0)?;
		return Ok(Compound {
			indexp: indexp,
			phasename: phasename.to_string(),
			h298: h298,
			s298: s298,
		});
	}
	
	pub fn load_endmembers(calculator: &Calculator, phasename: &str)->Result<Vec<Endmember>,ChemAppError>{
		let mut vecc : Vec<Endmember> = Vec::new();
		let indexp = calculator.engine.tqinp(phasename)?;
		let nendm = calculator.engine.tqnopc(indexp)?;
		for k in 0..nendm {
			let name = calculator.engine.tqgnpc(indexp, k+1)?;
			let h298 = calculator.engine.tqgdat(indexp, k+1, "H", 0)?;
			let s298 = calculator.engine.tqgdat(indexp, k+1, "S", 0)?;
			vecc.push(Endmember {
				indexp: indexp,
				indexc: k+1,
				phasename: phasename.to_string(),
				name: name,
				h298: h298,
				s298: s298,
			});
		}
		return Ok(vecc);
	}
	
	pub fn load_interactions_ge(calculator: &Calculator, phasename: &str)->Result<Vec<InteractionGEMQM>,ChemAppError>{
		let indexp = calculator.engine.tqinp(phasename)?;
		let sinteractions : Vec<String> = calculator.interactions_ge_expanded(indexp)?;
		//println!("sinderactions = {:?}", &sinteractions);
		let mut interactions : Vec<InteractionGEMQM> = Vec::new();
		for k in 0..sinteractions.len(){
			let values = [0.0f64;6];
			let values : Vec<f64> = calculator.engine.tqgpar(indexp,"G",k+1)?[0].clone();
			//println!("VALUES = {:?}", &values);
			interactions.push(InteractionGEMQM {
				indexp: indexp,
				index: k+1,
				phasename: phasename.to_string(),
				text: sinteractions[k].to_string(),
				values: values,
			});
		}
		return Ok(interactions);
	}
	
	pub fn load_interactions_magn(calculator: &Calculator, phase: &str)->Result<Vec<InteractionMagnMQM>,ChemAppError>{
		//todo!();
		return Ok(vec![]);
	}
	
	pub fn generate_lookup_cmp(compounds: &[Compound])->HashMap<String,usize>{
		let mut hmap : HashMap<String,usize> = HashMap::new();
		for k in 0..compounds.len(){
			let name = compounds[k].phasename.clone();
			hmap.insert(name,k);
		}
		return hmap;
	}
	
	pub fn generate_lookup_endm(endmembers: &[Endmember])->HashMap<(String,String),usize>{
		let mut hmap : HashMap<(String,String),usize> = HashMap::new();
		for k in 0..endmembers.len(){
			let phasename = endmembers[k].phasename.clone();
			let name = endmembers[k].name.clone();
			hmap.insert((phasename,name),k);
		}
		return hmap;
	}
	
	pub fn generate_lookup_interactions_ge(interactions: &[InteractionGEMQM])->HashMap<(String,String),usize>{
		let mut hmap : HashMap<(String,String),usize> = HashMap::new();
		for k in 0..interactions.len(){
			let phasename = interactions[k].phasename.clone();
			let text = interactions[k].text.clone();
			hmap.insert((phasename,text),k);
		}
		return hmap;
	}
	
	pub fn generate_lookup_interactions_magn(interactions: &[InteractionMagnMQM])->HashMap<(String,String),usize>{
		let mut hmap : HashMap<(String,String),usize> = HashMap::new();
		for k in 0..interactions.len(){
			let phasename = interactions[k].phasename.clone();
			let text = interactions[k].text.clone();
			hmap.insert((phasename,text),k);
		}
		return hmap;
	}
	
	pub fn new<T: AsRef<str> + std::fmt::Debug>(calculator: & Calculator, phasenames: &[T], include_ge: bool, include_magn: bool, include_endm: bool, include_cmp: bool)->Result<Self,ChemAppError> {
		let compounds : Vec<Compound> = Vec::new();
		let endmembers : Vec<Endmember> = Vec::new();
		let mut interactions_ge : Vec<InteractionGEMQM> = Vec::new();
		let interactions_magn : Vec<InteractionMagnMQM> = Vec::new();
		//println!("ParameterCache::new, phasenames = {:?}", &phasenames);
		for phasename in phasenames.iter(){
			let indexp = calculator.engine.tqinp(phasename.as_ref())?;
			let modelname = calculator.engine.tqmodl(indexp)?;
			//println!("modelname = {:?}", &modelname);
			match modelname.as_ref() {
				"PURE" => {
					if include_cmp {
						//let compound : Compound = Self::load_compound(calculator, phasename.as_ref())?;
						//compounds.push(compound);
					}
				}
				"SUBG" | "SUBQ" => {
					//println!("MATCH SUBG, SUBQ");
					if include_endm {
						//let mut endmembers_ : Vec<Endmember> = Self::load_endmembers(calculator, phasename.as_ref())?;
						//endmembers.append(&mut endmembers_);
					}
					if include_ge {
						let mut interactions : Vec<InteractionGEMQM> = Self::load_interactions_ge(calculator, phasename.as_ref())?;
						interactions_ge.append(&mut interactions);
					}
					if include_magn {
						todo!();
					}
				}
				_ => {
					// skip for now
				}
			}
		}
		let lookup_ge = Self::generate_lookup_interactions_ge(&interactions_ge);
		let lookup_magn  = Self::generate_lookup_interactions_magn(&interactions_magn);
		let lookup_cmp  = Self::generate_lookup_cmp(&compounds);
		let lookup_endm = Self::generate_lookup_endm(&endmembers);
		return Ok(Self {
			lookup_ge: lookup_ge,
			lookup_magn: lookup_magn,
			lookup_endm: lookup_endm,
			lookup_cmp: lookup_cmp,
			interactions_ge: interactions_ge,
			interactions_magn: interactions_magn,
			endmembers : endmembers,
			compounds: compounds,
		});
	}
	
	pub fn reset_all(&self, engine: &Engine)->Result<(),ChemAppError>{
		self.reset_compounds(engine)?;
		self.reset_endmembers(engine)?;
		self.reset_interactions_ge(engine)?;
		self.reset_interactions_magn(engine)?;
		return Ok(());
	}
	
	pub fn reset_compounds(&self, engine: &Engine)->Result<(),ChemAppError>{
		for k in 0..self.compounds.len(){
			self.compounds[k].reset(engine)?;
		}
		return Ok(());
	}
	
	pub fn reset_endmembers(&self, engine: &Engine)->Result<(),ChemAppError>{
		for k in 0..self.endmembers.len(){
			self.endmembers[k].reset(engine)?;
		}
		return Ok(());
	}
	
	pub fn reset_interactions_ge(&self, engine: &Engine)->Result<(),ChemAppError>{
		for k in 0..self.interactions_ge.len(){
			self.interactions_ge[k].reset(engine)?;
		}
		return Ok(());
	}
	
	pub fn reset_interactions_magn(&self, engine: &Engine)->Result<(),ChemAppError>{
		for k in 0..self.interactions_magn.len(){
			self.interactions_magn[k].reset(engine)?;
		}
		return Ok(());
	}
	
	pub fn set_interaction_ge(&self, engine: &Engine, phase: &str, interaction: &str, value: f64, tindex: usize, isdelta: bool)->Result<bool,ChemAppError>{
		//println!("LOOKING for ({:?},{:?}) in {:?}", phase, interaction, &self.lookup_ge);
		match self.lookup_ge.get(&(phase.to_string(),interaction.to_string())){
			Some(index) => {
				//println!("FOUND INTERACTION {:?} at {:?}", interaction, &index);
				let interaction = &self.interactions_ge[*index];
				let indexp = interaction.indexp;
				let indexi = interaction.index;
				let val = if isdelta {interaction.values[tindex] + value} else {value};
				engine.tqcdat(13,indexi,1,tindex+1,indexp,val)?;
				return Ok(true);
			}
			None => {return Ok(false);}
		}
		
	}
	
	pub fn set_interaction_magn(&self, engine: &Engine, phase: &str, interaction: &str, value: f64, tindex: usize, isdelta: bool)->Result<bool,ChemAppError>{
		todo!();
	}
	
	pub fn set_compound_h298(&self, engine: &Engine, phase: &str, value: f64, isdelta: bool)->Result<bool,ChemAppError> {
		match self.lookup_cmp.get(phase) {
			Some(index) => {
				let cmp = &self.compounds[*index];
				let indexp = cmp.indexp;
				let val = if isdelta {cmp.h298 + value} else {value};
				engine.tqcdat(1,0,0,1,indexp,val)?; // H298
				return Ok(true);
			}
			None => {return Ok(false);}
		}
	}
	
	pub fn set_compound_s298(&self, engine: &Engine, phase: &str, value: f64, isdelta: bool)->Result<bool,ChemAppError> {
		match self.lookup_cmp.get(phase) {
			Some(index) => {
				let cmp = &self.compounds[*index];
				let indexp = cmp.indexp;
				let val = if isdelta {cmp.s298 + value} else {value};
				engine.tqcdat(1,0,1,1,indexp,val)?; // S298
				return Ok(true);
			}
			None => {return Ok(false);}
		}
	}
	
	pub fn set_endmember_h298(&self, engine: &Engine, phase: &str, constituent: &str, value: f64, isdelta: bool)->Result<bool,ChemAppError> {
		match self.lookup_endm.get(&(phase.to_string(),constituent.to_string())) {
			Some(index) => {
				let endm = &self.endmembers[*index];
				let indexp = endm.indexp;
				let indexc = endm.indexc;
				let val = if isdelta {endm.h298 + value} else {value};
				engine.tqcdat(1,0,0,indexc,indexp,val)?; // H298
				return Ok(true);
			}
			None => {return Ok(false);}
		}
	}
	
	pub fn set_endmember_s298(&self, engine: &Engine, phase: &str, constituent: &str, value: f64, isdelta: bool)->Result<bool,ChemAppError> {
		match self.lookup_endm.get(&(phase.to_string(),constituent.to_string())) {
			Some(index) => {
				let endm = &self.endmembers[*index];
				let indexp = endm.indexp;
				let indexc = endm.indexc;
				let val = if isdelta {endm.s298 + value} else {value};
				engine.tqcdat(1,0,1,indexc,indexp,val)?; // S298
				return Ok(true);
			}
			None => {return Ok(false);}
		}
	}
	
}

impl Calculator {
	
	/// Creates an instance of `ParameterCache` for changing model parameter values and restoring them back to the original values
	pub fn generate_parameter_cache<T: AsRef<str> + std::fmt::Debug>(&mut self, phasenames: &[T], include_ge: bool, include_magn: bool, include_endm: bool, include_cmp: bool)->Result<(),ChemAppError> {
		self.cache = Some(ParameterCache::new(self, phasenames, include_ge, include_magn, include_endm, include_cmp)?);
		return Ok(());
	}
	
}