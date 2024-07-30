// chemapp::native.rs

/// Wraps around native ChemApp dll functions to adapt function calls to the Rust infrastructure

extern crate libloading;

use lazy_static::{LazyStatic};
use libloading::{Library, Symbol};
use std::str::{from_utf8};
use std::collections::{HashMap};
use std::cmp::{min};
use std::ffi::{CString, CStr};
use function_name::{named};

use crate::{Engine, SystemDimensions};
use crate::defs::{funcswin32};
use crate::error::{ChemAppError};

const NAME_LENGTH_MAX : usize = 25;

fn func_alias(name: &'static str)->&'static str {
	#[cfg(target_family="windows")]
	#[cfg(target_pointer_width="32")]
	return funcswin32[name];
	#[cfg(target_family="windows")]
	#[cfg(target_pointer_width="64")]
	return funcswin64[name];
}

fn clen(array: &[u8]) -> usize {
	let mut length = 0;
	for k in 0..array.len(){
		//print!("{} ", array[k]);
		if array[k] == 32{break;}
		else {length += 1;}
	}
	//println!("length is {}", length);
	return length;
}

fn wrap_result<T>(result: T, errcode: usize)->Result<T, ChemAppError>{
	match errcode {
		0 => Ok(result),
		_ => Err(ChemAppError::NativeError(errcode)),
	}
}


impl Engine {
	
	/******************************************/
	pub fn new(library_name: &str) -> Result<Engine,ChemAppError> {
		
		return Ok(Engine {
			n_isothermal: 0,
			n_target : 0,
			library_name: String::from(library_name),
			library: unsafe {Library::new(library_name)?},
		});
	}
	
	/******************************************/
	/// INITIALIZE-INTERFACE
	#[named]
	pub fn tqini(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
		let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
		func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-VERSION-NUMBER
	#[named]
	pub fn tqvers(&self) -> Result<i32, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut vers = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(vers: &mut i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut vers, &mut errcode);
		}
		return wrap_result(vers, errcode);
	}
	
	/******************************************/
	/// GET-COPYRIGHT-MESSAGE
	#[named]
	pub fn tqcprt(&self) -> Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CHECK-IF-CHEMAPP-LIGHT
	#[named]
	pub fn tqlite(&self) -> Result<bool, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut lite = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(lite: &mut i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut lite, &mut errcode);
		}
		return wrap_result(lite > 0, errcode);
	}
	
	/******************************************/
	/// GET-USER-ID
	#[named]
	pub fn tqgtid(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; 256] = [0;256];
		unsafe {
			let func : Symbol<extern "stdcall" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], 256, &mut errcode);
		}
		return wrap_result(from_utf8(&cstring[0..clen(&cstring)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-USER-NAME
	#[named]
	pub fn tqgtnm(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; 80] = [0;80];
		unsafe {
			let func : Symbol<extern "stdcall" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], 80, &mut errcode);
		}
		return wrap_result(from_utf8(&cstring[0..clen(&cstring)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-PROGRAM-ID
	#[named]
	pub fn tqgtpi(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; 80] = [0;80];
		unsafe {
			let func : Symbol<extern "stdcall" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], 80, &mut errcode);
		}
		return wrap_result(from_utf8(&cstring[0..clen(&cstring)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-HASP-DONGLE-INFO
	#[named]
	pub fn tqgthi(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; 80] = [0;80];
		unsafe {
			let func : Symbol<extern "stdcall" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], 80, &mut errcode);
		}
		return wrap_result(from_utf8(&cstring[0..clen(&cstring)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-EXPIRATION-MONTH-AND-YEAR
	#[named]
	pub fn tqgted(&self)->Result<(u32,u32), ChemAppError>{
		let fname = func_alias(function_name!());
		todo!();
		let mut month : u32 = 0;
		let mut year  : u32 = 0;
		let mut errcode = 0;
		return wrap_result((month, year), errcode);
	}
	
	/******************************************/
	/// SET-CONFIGURATION-OPTION
	#[named]
	pub fn tqconf(&self, option: &str, valuea: usize, valueb: usize, valuec: usize)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption : CString = CString::new(option)?;
		let coption_length = option.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(coption: &u8, coption_length: usize, valuea: &usize, valueb: &usize, valuec: &usize, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], coption_length, &valuea, &valueb, &valuec, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-ARRAY-SIZES
	#[named]
	pub fn tqsize(&self)->Result<SystemDimensions, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut dims : SystemDimensions =  SystemDimensions::new();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		return wrap_result(dims, errcode);
	}
	
	/******************************************/
	/// GET-CURRENT-DIMENSIONS
	#[named]
	pub fn tqused(&self)->Result<SystemDimensions, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut dims : SystemDimensions =  SystemDimensions::new();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		return wrap_result(dims, errcode);
	}
	
	/******************************************/
	/// GET-VALUE-OF-INPUT-OUTPUT-OPTION
	#[named]
	pub fn tqgio(&self, option: &str)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut num = 0;
		let coption: CString = CString::new(option)?;
		unsafe {
			let func : Symbol<extern "stdcall" fn(option: &u8, option_len: usize, num: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &mut num, &mut errcode);
		}
		return wrap_result(num, errcode);
	}
	
	/******************************************/
	/// CHANGE-INPUT-OPTION
	#[named]
	pub fn tqcio(&self, option: &str, unit: usize)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, unit: &usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// READ-DATA-FILE
	#[named]
	pub fn tqrfil(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// READ-BINARY-DATA-FILE
	#[named]
	pub fn tqrbin(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// READ-TRANSPARENT-DATA-FILE
	#[named]
	pub fn tqrcst(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// OPEN-FILE
	#[named]
	pub fn tqopen(&self, filename: &str, unit: i32)->Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let cfilename: CString = CString::new(filename)?;
		let cfilename_length = filename.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(cfilename: &u8, filename_length: usize, unit: &i32, errcode: &mut usize)>
			= self.library.get(fname.as_bytes())?;
			func(&cfilename.as_bytes()[0], cfilename_length, &unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// TODO WRITE-STRING
	#[named]
	pub fn tqwstr(&self, option: &str, text: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let ctext : CString = CString::new(text)?;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, text: &u8, text_len: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &ctext.as_bytes()[0], text.len(), &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// OPEN-ASCII-DATA-FILE
	#[named]
	pub fn tqopna(&self, name: &str, unit: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(cname: &u8, cfilename_length: usize, unit: &i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// OPEN-BINARY-DATA-FILE
	#[named]
	pub fn tqopnb(&self, name: &str, unit: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(cname: &u8, cname_length: usize, unit: &i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// OPEN-TRANSPARENT-DATA-FILE
	#[named]
	pub fn tqopnt(&self, name: &str, unit: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(cname: &u8, cname_length: usize, unit: &i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CLOSE-FILE
	#[named]
	pub fn tqclos(&self, unit: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(unit: &i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&unit, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// TODO
	pub fn tqgtrh(&self)->Result<(),i32>{
		todo!();
	}
	
	/******************************************/
	/// GET-SYSTEM-UNIT
	#[named]
	pub fn tqgsu(&self, option: &str) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len()-1;
		let mut cunit: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, unit: &mut u8, unit_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &mut cunit[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cunit[0..clen(&cunit)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// CHANGE-SYSTEM-UNIT
	#[named]
	pub fn tqcsu(&self, option: &str, unit: &str) -> Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let cunit:   CString = CString::new(unit)?;
		let option_length = option.len();
		let unit_length = unit.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, unit: &u8, unit_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &cunit.as_bytes()[0], unit_length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-INDEX-NUMBER-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqinsc(&self, name: &str) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let name_length = name.len();
		let mut indexs = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(name: &u8, name_length: usize, indexs: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], name_length, &mut indexs, &mut errcode);
		}
		return wrap_result(indexs, errcode);
	}
	
	/******************************************/
	/// GET-NAME-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqgnsc(&self, indexs: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexs: &usize, name: &mut u8, name_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexs, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cname[0..clen(&cname)])?.replace('\0', "").to_owned(), errcode);
	}
	
	/******************************************/
	/// CHANGE-NAME-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqcnsc(&self, indexs: usize, name: &str) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let name_length = name.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexs: &usize, name: &u8, name_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexs, &cname.as_bytes()[0], name_length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-NUMBER-OF-SYSTEM-COMPONENTS
	#[named]
	pub fn tqnosc(&self) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nscom = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(nscom: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut nscom, &mut errcode);
		}
		return wrap_result(nscom, errcode);
	}
	
	/******************************************/
	/// GET-STOICHIOMETRY-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqstsc(&self,indexs: usize)->Result<(Vec<f64>,f64),ChemAppError>{
		let fname = func_alias(function_name!());
		let ncomp = self.tqnosc()?;
		let mut stoi : Vec<f64> = vec![0.0;ncomp];
		let mut wmass = 0.0f64;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexs: &usize, stoi: &mut f64, wmass: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&ncomp, &mut stoi[0], &mut wmass, &mut errcode);
		}
		println!("indexs = {}, stoi = {:?}", indexs, &stoi);
		return wrap_result((stoi, wmass), errcode);
	}
	
	/******************************************/
	/// CHANGE-SYSTEM-COMPONENTS
	#[named]
	pub fn tqcsc(&self, names: &[&str])->Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let nsyscom = self.tqnosc()?;
		let length: usize = 24;
		let mut namememory : Vec::<u8> = vec![32; (nsyscom+1)*length];
		let mut cname: CString;
		let mut cbytes: &[u8];
		let mut size: usize;
		for k in 0..names.len(){
			cname = CString::new(names[k])?;
			cbytes = cname.as_bytes();
			size = min(length, cbytes.len());
			namememory[k*length..k*length+size].clone_from_slice(&cbytes[0..size]);
			//namememory[(k+1)*length] = 0;
		}
		for k in names.len()..nsyscom{
			cname = CString::new(self.tqgnsc(k+1)?)?;
			cbytes = cname.as_bytes();
			size = min(length, cbytes.len());
			namememory[k*length..k*length+size].clone_from_slice(&cbytes[0..size]);
			//namememory[(k+1)*length] = 0;
		}
		//println!("namememory = {:?}", namememory);
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(names: &u8, names_length: usize, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&namememory[0], length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-INDEX-NUMBER-OF-PHASE
	#[named]
	pub fn tqinp(&self, name: &str) -> Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut indexp  = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(cname: &u8, cname_length: usize, indexp: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &mut indexp, &mut errcode);
		}
		return wrap_result(indexp, errcode);
	}
	
	/******************************************/
	/// GET-NAME-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgnp(&self, indexp: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cname[0..clen(&cname)])?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-MODEL-NAME-OF-PHASE
	#[named]
	pub fn tqmodl(&self, indexp: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cname)?.to_owned(), errcode);
	}
	
	/******************************************/
	/// GET-NUMBER-OF-PHASES
	#[named]
	pub fn tqnop(&self) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nphase  = 0;
			let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(nphase: &mut usize, errcode: &mut usize)>
				= self.library.get(fname.as_bytes())?;
			func(&mut nphase, &mut errcode);
		}
		return wrap_result(nphase, errcode);
	}
	
	/******************************************/
	/// GET-INDEX-NUMBER-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqinpc(&self, indexp: usize, name: &str)-> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut indexc  = 0;
		let mut errcode = 0;
		let cname : CString = CString::new(name)?;
		let cname_length = name.len();
		unsafe {
			let func: Symbol<extern "stdcall" fn(cname: &u8, name_length: usize, indexp: &usize, indexc: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &indexp, &mut indexc, &mut errcode);
		}
		return wrap_result(indexc, errcode);
	}
	
	/******************************************/
	/// GET-NAME-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgnpc(&self, indexp: usize, indexc: usize)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cname)?.to_owned(), errcode);
	}
	
	/******************************************/
	/// PHASE-CONSTITUENT-IS-INCOMING-SPECIES
	#[named]
	pub fn tqpcis(&self, indexp: usize, indexc: usize)->Result<bool,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut value = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, value: &mut i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut value, &mut errcode);
		}
		return wrap_result(value > 0, errcode);
	}
	
	/******************************************/
	/// GET-NUMBER-OF-PHASE-CONSTITUENTS
	#[named]
	pub fn tqnopc(&self, indexp: usize)->Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nconst  = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, nconst: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nconst, &mut errcode);
		}
		return wrap_result(nconst, errcode);
	}
	
	/******************************************/
	/// TODO
	#[named]
	pub fn tqstpc(&self)->Result<(),ChemAppError>{
		todo!();
	}
	
	/******************************************/
	/// GET-CHARGE-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqchar(&self, indexp: usize, indexc: usize)->Result<i32, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut charge = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, charge: &mut i32, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut charge, &mut errcode);
		}
		return wrap_result(charge, errcode);
	}
	
	/******************************************/
	/// GET-INDEX-NUMBER-OF-SUBLATTICE-CONSTITUENT
	#[named]
	pub fn tqinlc(&self, name: &str, indexp: usize, indexl: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let mut errcode = 0;
		let mut indexc: usize = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(name: &u8, name_len: usize, indexp: &usize, indexl: &usize, indexc: &mut usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], name.len(), &indexp, &indexl, &mut indexc, &mut errcode);
		}
		return wrap_result(indexc, errcode);
	}
	
	/******************************************/
	///TODO
	#[named]
	pub fn tqgnlc(&self)->Result<(),ChemAppError>{
		todo!();
	}
	
	/******************************************/
	/// GET-NUMBER-OF-SUBLATTICES
	#[named]
	pub fn tqnosl(&self, indexp: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut nosl: usize = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, nosl: &mut usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nosl, &mut errcode);
		}
		return wrap_result(nosl, errcode);
	}
	
	/******************************************/
	/// GET-NUMBER-OF-SUBLATTICE-SPECIES
	#[named]
	pub fn tqnolc(&self, indexp: usize, index: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut nosc = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, index: &usize, nosc: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &index, &mut nosc, &mut errcode);
		}
		return wrap_result(nosc, errcode);
	}
	
	/******************************************/
	/// GET-STATUS-OF-PHASE
	#[named]
	pub fn tqgsp(&self, indexp: usize)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cstatus: [u8;NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, cstatus: &mut u8, cstatus_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cstatus[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cstatus)?.to_owned(), errcode);
	}
	
	/******************************************/
	/// CHANGE-STATUS-OF-PHASE
	#[named]
	pub fn tqcsp(&self, indexp: usize, status: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cstatus: CString = CString::new(status)?;
		let cstatus_length = status.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, cstatus: &u8, cstatus_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &cstatus.as_bytes()[0], cstatus_length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-STATUS-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgspc(&self, indexp: usize, indexc: usize)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cstatus: [u8;NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, cstatus: &u8, cstatus_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cstatus[0], NAME_LENGTH_MAX, &mut errcode);
		}
		return wrap_result(from_utf8(&cstatus)?.to_owned(), errcode);
	}
	
	/******************************************/
	/// CHANGE-STATUS-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqcspc(&self, indexp: usize, indexc: usize, status: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cstatus: CString = CString::new(status)?;
		let cstatus_length = status.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, status: &u8, status_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &cstatus.as_bytes()[0], cstatus_length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// SET-EQUILIBRIUM-CONDITION
	#[named]
	pub fn tqsetc(&self, option: &str, indexp: usize, indexc: usize, val: f64) -> Result<i32,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut numcon  = 0;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, val: &f64, numcon: &mut i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &val, &mut numcon, &mut errcode);
		}
		return wrap_result(numcon, errcode);
	}
	
	/******************************************/
	/// REMOVE-EQUILIBRIUM-CONDITION
	#[named]
	pub fn tqremc(&self, numcon: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(numcon: &i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&numcon, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// SET-NAME-TEMPERATURE-PRESSURE-FOR-A-STREAM
	#[named]
	pub fn tqsttp(&self, idents: &str, vals: (f64,f64))->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let vals_ = [vals.0, vals.1];
		let cidents: CString = CString::new(idents)?;
		unsafe {
			let func: Symbol<extern "stdcall" fn(idents: &u8, idents_len: usize, vals: &f64, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &vals_[0], &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// SET-CONSTITUENT-AMOUNTS-FOR-A-STREAM
	#[named]
	pub fn tqstca(&self, idents: &str, indexp: usize, indexc: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cidents: CString = CString::new(idents)?;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(idents: &u8, idents_len: usize, indexp: &usize, indexc: &usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &indexp, &indexc, &val, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// SET-EQUILIBRIUM-CONDITION-WHEN-STREAM-INPUT
	#[named]
	pub fn tqstec(&self, option: &str, indexp: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, indexp: &usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &val, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// REMOVE-STREAM
	#[named]
	pub fn tqstrm(&self, idents: &str)->Result<(),ChemAppError>{
		//todo!();
		let fname = func_alias(function_name!());
		let cidents: CString = CString::new(idents)?;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(idents: &u8, idents_len: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CALCULATE-EQUILIBRIUM
	#[named]
	pub fn tqce(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CALCULATE-EQUILIBRIUM-AND-LIST-RESULTS
	#[named]
	pub fn tqcel(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS
	#[named]
	pub fn tqcen(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
			let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS-AND-LIST-RESULTS
	#[named]
	pub fn tqcenl(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP
	#[named]
	pub fn tqmap(&self, option: &str, indexp: usize, indexc: usize, vals: (f64,f64))->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		let mut icont : usize = 0;
		let vals_: [f64;2] = [vals.0, vals.1];
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &indexc, &vals_[0], &mut icont, &mut errcode);
		}
		return wrap_result(icont, errcode);
	}
	
	/******************************************/
	// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP-AND-LIST-RESULTS
	#[named]
	pub fn tqmapl(&self, option: &str, indexp: usize, indexc: usize, vals: (f64,f64))->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		let mut icont : usize = 0;
		let vals_: [f64;2] = [vals.0, vals.1];
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &indexc, &vals_[0], &mut icont, &mut errcode);
		}
		return wrap_result(icont, errcode);
	}
	
	/******************************************/
	/// CHANGE-LIMIT-OF-TARGET-VARIABLE
	#[named]
	pub fn tqclim(&self, option: &str, val: f64) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &val, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// SHOW-PRESENT-SETTINGS
	#[named]
	pub fn tqshow(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-RESULT
	#[named]
	pub fn tqgetr(&self, option: &str, indexp: usize, indexc: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut value = 0.0f64;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, value: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &mut value, &mut errcode);
		}
		return wrap_result(value, errcode);
	}
	
	/******************************************/
	/// GET-PROPERTY-OF-A-PHASE-CONSTITUENT
	#[named]
	pub fn tqgdpc(&self, option: &str, indexp: usize, index: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut fval = 0.0f64;
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(option: &u8, option_len: usize, indexp: &usize, index: &usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &index, &mut fval, &mut errcode);
		}
		return wrap_result(fval, errcode);
		
	}
	
	/******************************************/
	/// GET-THERMODYNAMIC-PROPERTY-OF-A-STREAM
	#[named]
	pub fn tqstxp(&self, idents: &str, option: &str)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let cidents : CString = CString::new(idents)?;
		let coption : CString = CString::new(option)?;
		let mut errcode = 0;
		let mut fval = 0.0f64;
		unsafe {
			let func : Symbol<extern "stdcall" fn(idents: &u8, idents_len: usize, option: &u8, option_len: usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &coption.as_bytes()[0], option.len(), &mut fval, &mut errcode);
		}
		return wrap_result(fval, errcode);
	}
	
	/******************************************/
	/// GET-CALCULATED-EQUILIBRIUM-SUBLATTICE-SITE-FRACTION
	#[named]
	pub fn tqgtlc(&self, indexp: usize, indexl: usize, indexc: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut fval = 0.0f64;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexl: &usize, indexc: &usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexl, &indexc, &mut fval, &mut errcode);
		}
		return wrap_result(fval, errcode);
	}
	
	/******************************************/
	/// GET-CALCULATED-QUADRUPLET-OR-PAIR-FRACTION
	#[named]
	pub fn tqbond(&self, indexp: usize, indexa: usize, indexb: usize, indexc: usize, indexd: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut value = 0.0;
			let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(indexp: &usize, indexa: &usize, indexb: &usize, indexc: &usize, indexd: &usize, value: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexa, &indexb, &indexc, &indexd, &mut value, &mut errcode);
		}
		return wrap_result(value, errcode);
	}
	
	/******************************************/
	/// GET-ERROR-MESSAGE
	#[named]
	pub fn tqerr(&self)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// GET-INPUT-THERMODYNAMIC-DATA-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgdat(&self, indexp: usize, indexc: usize, option: &str, indexr: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut fval = 0.0f64;
		let coption: CString = CString::new(option)?;
		unsafe {
			let func : Symbol<extern "stdcall" fn(indexp: &usize, indexc: &usize, option: &u8, option_len: usize, indexr: &usize, fval: &mut f64, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &coption.as_bytes()[0], option.len(), &indexr, &mut fval, &mut errcode);
		}
		return wrap_result(fval, errcode);
	}
	
	/******************************************/
	/// TODO
	#[named]
	pub fn tqlpar(&self)->Result<(),ChemAppError>{
		todo!();
	}
	
	/******************************************/
	/// TODO
	#[named]
	pub fn tqgpar(&self)->Result<(),ChemAppError>{
		todo!();
	}
	
	/******************************************/
	/// CHANGES-DATA-OF-THERMODYNAMIC-DATA-FILE
	#[named]
	pub fn tqcdat(&self, i1: usize, i2: usize, i3: usize, i4: usize, i5: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		unsafe {
			let func: Symbol<extern "stdcall" fn(i1: &usize, i2: &usize, i3: &usize, i4: &usize, i5: &usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&i1, &i2, &i3, &i4, &i5, &val, &mut errcode);
		}
		return wrap_result((), errcode);
	}
	
	/******************************************/
	/// WRITE-DATA-FILE-IN-ASCII-FORMAT
	#[named]
	pub fn tqwasc(&self, file: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cfile = CString::new(file)?;
		let cfile_length = file.len();
		let mut errcode = 0;
		unsafe{
			let func: Symbol<extern "stdcall" fn(cfile: &u8, cfile_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cfile.as_bytes()[0], cfile_length, &mut errcode);
		}
		return wrap_result((), errcode);
	}
}