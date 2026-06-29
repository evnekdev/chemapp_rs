// chemapp_rs::parse.rs

use nom::{IResult,sequence::{tuple,delimited},
branch::{alt},
character::complete::{char,multispace1,multispace0,digit1},
combinator::{map_res},
bytes::complete::{tag},
multi::{separated_list1}
};

use crate::Engine;

/********************************************************************************************************/
/********************************************************************************************************/

fn parse_speciespower<'a>(s: &'a str)->IResult<&'a str,(usize,&'a str)>{
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

pub fn convert_magn_interaction_species<'a>(engine: &'a Engine, indexp: usize, s: &'a str)->IResult<&'a str, Vec<String>>{
	todo!();
}

pub fn convert_ge_interaction_species<'a>(engine: &'a Engine, indexp: usize, s: &'a str)->IResult<&'a str,Vec<String>>{
	match tuple(( parse_interaction_index, separated_list1(char('-'), parse_speciespower), multispace1, tag(":"), multispace1, parse_ending_species, multispace1, parse_interaction_type  ))(s) {
		Ok((s, ((index,nterms), vecc, _, _, _, species0, _, itype))) => {
			let nspecies1 = engine.tqnolc(indexp, 1).unwrap();
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
	let vecc_ : Vec<String> = vec![species_name(engine,indexp,species1),species_name(engine,indexp,species2),species_name(engine,indexp,species3),species_name(engine,indexp,species4)];
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
	match parse_interaction(s) {
		Ok((s, ((index,nterms), vecc, _, _, _, species0, _, itype))) => {
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

pub fn convert_magn_interaction<'a>(engine: &'a Engine, indexp: usize, s: &'a str)->IResult<&'a str, String>{
	todo!();
}
