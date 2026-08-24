// chemapp::native.rs

//! This submodule exports ChemApp functions as-is with the minimal changes in the function signatures to adapt to the Rust infrastructure.
#![allow(unused_imports)]

extern crate libloading;

use libloading::{Library, Symbol};
use std::str::{from_utf8};
use std::cmp::{min};
use std::ffi::{CString};
use function_name::{named};

use crate::DEFAULT_LIBNAME;
use crate::{SystemDimensions, TransparentHeader};
use crate::defs::{FUNCSWIN32,FUNCSWIN64,FUNCSUNIX32,FUNCSUNIX64};
use crate::error::{ChemAppError};

const NAME_LENGTH_MAX : usize = 25;
const TQGTID_CHARACTER_LENGTH: usize = 255;
const TQGTNM_CHARACTER_LENGTH: usize = 80;
const TQGTRH_PROGRAM_NAME_LENGTH: usize = 40;
const TQGTRH_USER_ID_LENGTH: usize = 255;
const TQGTRH_TEXT_LENGTH: usize = 80;
const TQERR_RECORD_LENGTH: usize = 80;
const TQERR_RECORD_COUNT: usize = 3;

/*********************************************************************************************************************************************************************************************************/
/*********************************************************************************************************************************************************************************************************/

fn func_alias(name: &'static str)->&'static str {
	 #[cfg(all(target_family = "windows", target_pointer_width = "32"))]
    let funcs = &FUNCSWIN32;

    #[cfg(all(target_family = "windows", target_pointer_width = "64"))]
    let funcs = &FUNCSWIN64;

    #[cfg(all(target_family = "unix", target_pointer_width = "32"))]
    let funcs = &FUNCSUNIX32;

    #[cfg(all(target_family = "unix", target_pointer_width = "64"))]
    let funcs = &FUNCSUNIX64;

    funcs[name]
}

/// Converts exactly one fixed-width Fortran CHARACTER result to a Rust string.
///
/// `character_length` is the Fortran declaration, not necessarily the Rust
/// allocation length. ChemApp may blank-pad the record and is not required to
/// NUL-terminate it, so this deliberately never examines a convenience byte
/// beyond the declared record. Internal spaces are data: only trailing
/// Fortran padding is removed. Fixed-width output must never be parsed by
/// looking for its first space.
fn fixed_fortran_string(buffer: &[u8], character_length: usize) -> Result<String, ChemAppError> {
	if buffer.len() < character_length {
		return Err(ChemAppError::OtherError(format!(
			"Fortran CHARACTER buffer has length {}, but {} bytes are required",
			buffer.len(), character_length
		)));
	}

	let record = &buffer[..character_length];
	let nul_end = record.iter().position(|byte| *byte == 0).unwrap_or(record.len());
	let end = record[..nul_end]
		.iter()
		.rposition(|byte| *byte != b' ')
		.map(|last| last + 1)
		.unwrap_or(0);
	return Ok(from_utf8(&record[..end])?.to_owned());
}

/// Converts TQERR's three 80-character Fortran records without treating the
/// contiguous 240-byte Rust allocation as a single CHARACTER*240 value.
fn tqerr_message(buffer: &[u8]) -> Result<String, ChemAppError> {
	if buffer.len() < TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT {
		return Err(ChemAppError::OtherError("TQERR buffer is too short".to_owned()));
	}

	let mut records = Vec::new();
	for record in buffer[..TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT].chunks_exact(TQERR_RECORD_LENGTH) {
		let text = fixed_fortran_string(record, TQERR_RECORD_LENGTH)?;
		if !text.is_empty() {
			records.push(text);
		}
	}
	return Ok(records.join("\n"));
}

fn cstring_character_length(cstring: &CString) -> usize {
	// CString::as_bytes deliberately excludes the trailing NUL.
	return cstring.as_bytes().len();
}

fn wrap_result<T>(result: T, errcode: usize)->Result<T, ChemAppError>{
	match errcode {
		0 => Ok(result),
		_ => Err(ChemAppError::NativeError(errcode)),
	}
}

/*********************************************************************************************************************************************************************************************************/
/*********************************************************************************************************************************************************************************************************/

/// An encapsulation of a single loaded DLL - different instances correspond to different DLLs. ChemApp tq... functions are exported as methods, rather than independent functions to support multiple DLL loading.
#[derive(Debug)]
pub struct Engine {
	pub n_isothermal: usize,
	pub n_target: usize,
	pub(crate) library_name: String,
	library: Library,
}

impl Default for Engine {
	fn default()->Engine {
		return Engine::new(DEFAULT_LIBNAME).unwrap();
	}
}

/*********************************************************************************************************************************************************************************************************/
/*********************************************************************************************************************************************************************************************************/

impl Engine {
	
	/*****************************************************************************************************************************************************************************************************/
	/// Initializes a new instance of `Engine` from a DLL path or name. In case a name only is used, the DLL has to be discoverable in PATH system variable (modify the system environment variables if it is not the case).
	pub fn new(library_name: &str) -> Result<Engine,ChemAppError> {
		
		return Ok(Engine {
			n_isothermal: 0,
			n_target : 0,
			library_name: String::from(library_name),
			library: unsafe {Library::new(library_name)?},
		});
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// INITIALIZE-INTERFACE
	#[named]
	pub fn tqini(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-VERSION-NUMBER
	#[named]
	pub fn tqvers(&self) -> Result<i32, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut vers = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(vers: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut vers, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(vers: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut vers, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(vers, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-COPYRIGHT-MESSAGE
	#[named]
	pub fn tqcprt(&self) -> Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHECK-IF-CHEMAPP-LIGHT
	#[named]
	pub fn tqlite(&self) -> Result<bool, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut lite = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(lite: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut lite, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(lite: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut lite, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(lite > 0, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-USER-ID
	#[named]
	pub fn tqgtid(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		// The GTT bridge declares CHARACTER*255. No spare Rust byte is passed
		// to Fortran as part of that declaration.
		let mut cstring: [u8; TQGTID_CHARACTER_LENGTH] = [0; TQGTID_CHARACTER_LENGTH];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], TQGTID_CHARACTER_LENGTH, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cstring: &mut u8, errcode: &mut usize, length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], &mut errcode, TQGTID_CHARACTER_LENGTH);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cstring, TQGTID_CHARACTER_LENGTH)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-USER-NAME
	#[named]
	pub fn tqgtnm(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; TQGTNM_CHARACTER_LENGTH] = [0; TQGTNM_CHARACTER_LENGTH];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], TQGTNM_CHARACTER_LENGTH, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cstring: &mut u8, errcode: &mut usize, length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], &mut errcode, TQGTNM_CHARACTER_LENGTH);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cstring, TQGTNM_CHARACTER_LENGTH)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-PROGRAM-ID
	#[named]
	pub fn tqgtpi(&self)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut cstring: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(cstring: &mut u8, length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cstring: &mut u8, errcode: &mut usize, length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cstring, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-HASP-DONGLE-INFO
	#[named]
	pub fn tqgthi(&self)->Result<(String,i32), ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut hid = 0;
		let mut cstring: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(cstring: &mut u8, length: usize, hid: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], NAME_LENGTH_MAX, &mut hid, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cstring: &mut u8, hid: &mut i32, errcode: &mut usize, length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cstring[0], &mut hid, &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result((fixed_fortran_string(&cstring, NAME_LENGTH_MAX)?,hid), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-EXPIRATION-MONTH-AND-YEAR
	#[named]
	pub fn tqgted(&self)->Result<(u32,u32), ChemAppError>{
		let fname = func_alias(function_name!());
		let mut month : u32 = 0;
		let mut year  : u32 = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(month: &mut u32, year: &mut u32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut month, &mut year, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(month: &mut u32, year: &mut u32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut month, &mut year, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((month, year), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SET-CONFIGURATION-OPTION
	#[named]
	pub fn tqconf(&self, option: &str, valuea: usize, valueb: usize, valuec: usize)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption : CString = CString::new(option)?;
		let coption_length = option.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(coption: &u8, coption_length: usize, valuea: &usize, valueb: &usize, valuec: &usize, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], coption_length, &valuea, &valueb, &valuec, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(coption: &u8, valuea: &usize, valueb: &usize, valuec: &usize, errcode: &mut usize, coption_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &valuea, &valueb, &valuec, &mut errcode, coption_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-ARRAY-SIZES
	#[named]
	pub fn tqsize(&self)->Result<SystemDimensions, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut dims : SystemDimensions =  SystemDimensions::new();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(dims, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-CURRENT-DIMENSIONS
	#[named]
	pub fn tqused(&self)->Result<SystemDimensions, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut dims : SystemDimensions =  SystemDimensions::new();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(na: &mut i32, nb: &mut i32, nc: &mut i32, nd: &mut i32, ne: &mut i32, nf: &mut i32, ng: &mut i32, nh: &mut i32, ni: &mut i32, nj: &mut i32, nk: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut dims.nconstituents, &mut dims.ncomponents, &mut dims.nmixtures, &mut dims.nexcess_gibbs, &mut dims.nexcess_magnetic, &mut dims.nsublattices, &mut dims.nspecies, &mut dims.nconstituents_mqm, &mut dims.nranges_constituent, &mut dims.nranges, &mut dims.ndependent, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(dims, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-VALUE-OF-INPUT-OUTPUT-OPTION
	#[named]
	pub fn tqgio(&self, option: &str)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut num = 0;
		let coption: CString = CString::new(option)?;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(option: &u8, option_len: usize, num: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &mut num, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, num: &mut usize, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &mut num, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result(num, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-INPUT-OPTION
	#[named]
	pub fn tqcio(&self, option: &str, unit: usize)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, unit: &usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, unit: &usize, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &unit, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// READ-DATA-FILE
	#[named]
	pub fn tqrfil(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// READ-BINARY-DATA-FILE
	#[named]
	pub fn tqrbin(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// READ-TRANSPARENT-DATA-FILE
	#[named]
	pub fn tqrcst(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()>
			= self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// OPEN-FILE
	#[named]
	pub fn tqopen(&self, filename: &str, unit: usize)->Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let cfilename: CString = CString::new(filename)?;
		let cfilename_length = filename.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cfilename: &u8, filename_length: usize, unit: &usize, errcode: &mut usize)>
			= self.library.get(fname.as_bytes())?;
			func(&cfilename.as_bytes()[0], cfilename_length, &unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cfilename: &u8, unit: &usize, errcode: &mut usize, filename_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cfilename.as_bytes()[0], &unit, &mut errcode, cfilename_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// TODO WRITE-STRING
	#[named]
	pub fn tqwstr(&self, option: &str, text: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let ctext : CString = CString::new(text)?;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, text: &u8, text_len: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &ctext.as_bytes()[0], text.len(), &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, text: &u8, errcode: &mut usize, option_len: usize, text_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &ctext.as_bytes()[0], &mut errcode, option.len(), text.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// OPEN-ASCII-DATA-FILE
	#[named]
	pub fn tqopna(&self, name: &str, unit: usize) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cname: &u8, cfilename_length: usize, unit: &usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cname: &u8, unit: &usize, errcode: &mut usize, cfilename_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &unit, &mut errcode, cname_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// OPEN-BINARY-DATA-FILE
	#[named]
	pub fn tqopnb(&self, name: &str, unit: usize) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cname: &u8, cname_length: usize, unit: &usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cname: &u8, unit: &usize, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &unit, &mut errcode, cname_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// OPEN-TRANSPARENT-DATA-FILE
	#[named]
	pub fn tqopnt(&self, name: &str, unit: usize) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cname: &u8, cname_length: usize, unit: &usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cname: &u8, unit: &usize, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &unit, &mut errcode, cname_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CLOSE-FILE
	#[named]
	pub fn tqclos(&self, unit: usize) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(unit: &usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&unit, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(unit: &usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&unit, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-TRANSPARENT-FILE-HEADER-INFO
	#[named]
	pub fn tqgtrh(&self)->Result<TransparentHeader,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cver = 0;
		let mut cnwp : [u8; TQGTRH_PROGRAM_NAME_LENGTH] = [0; TQGTRH_PROGRAM_NAME_LENGTH];
		let mut cvnw : [i32; 3] = [0; 3];
		let mut cnrp : [u8; TQGTRH_PROGRAM_NAME_LENGTH] = [0; TQGTRH_PROGRAM_NAME_LENGTH];
		let mut cvnr : [i32; 3] = [0; 3];
		let mut cdtc : [i32; 6] = [0; 6];
		let mut cdte : [i32; 6] = [0; 6];
		let mut cid  : [u8; TQGTRH_USER_ID_LENGTH] = [0; TQGTRH_USER_ID_LENGTH];
		let mut cusr : [u8; TQGTRH_TEXT_LENGTH] = [0; TQGTRH_TEXT_LENGTH];
		let mut crem : [u8; TQGTRH_TEXT_LENGTH] = [0; TQGTRH_TEXT_LENGTH];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(&mut i32, &mut u8, usize, &mut i32, &mut u8, usize, &mut i32, &mut i32, &mut i32, &mut u8, usize, &mut u8, usize, &mut u8, usize, &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cver, &mut cnwp[0], TQGTRH_PROGRAM_NAME_LENGTH, &mut cvnw[0], &mut cnrp[0], TQGTRH_PROGRAM_NAME_LENGTH, &mut cvnr[0], &mut cdtc[0], &mut cdte[0], &mut cid[0], TQGTRH_USER_ID_LENGTH, &mut cusr[0], TQGTRH_TEXT_LENGTH, &mut crem[0], TQGTRH_TEXT_LENGTH, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(&mut i32, &mut u8, &mut i32, &mut u8, &mut i32, &mut i32, &mut i32, &mut u8, &mut u8, &mut u8, &mut usize, usize, usize, usize, usize, usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cver, &mut cnwp[0], &mut cvnw[0], &mut cnrp[0], &mut cvnr[0], &mut cdtc[0], &mut cdte[0], &mut cid[0], &mut cusr[0], &mut crem[0], &mut errcode, TQGTRH_PROGRAM_NAME_LENGTH, TQGTRH_PROGRAM_NAME_LENGTH, TQGTRH_USER_ID_LENGTH, TQGTRH_TEXT_LENGTH, TQGTRH_TEXT_LENGTH);
		}
		/******************************************************************************************************/
		let header : TransparentHeader = TransparentHeader {
			version : cver,
			name_writing_program       : fixed_fortran_string(&cnwp, TQGTRH_PROGRAM_NAME_LENGTH)?,
			version_writing_program    : cvnw,
			name_reading_program       : fixed_fortran_string(&cnrp, TQGTRH_PROGRAM_NAME_LENGTH)?,
			minversion_reading_program : cvnr,
			creation_date              : cdtc,
			expiry_date                : cdte,
			user_ids_allowed           : fixed_fortran_string(&cid, TQGTRH_USER_ID_LENGTH)?,
			license_holders_allowed    : fixed_fortran_string(&cusr, TQGTRH_TEXT_LENGTH)?,
			remark                     : fixed_fortran_string(&crem, TQGTRH_TEXT_LENGTH)?,
		};
		return wrap_result(header, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-SYSTEM-UNIT
	#[named]
	pub fn tqgsu(&self, option: &str) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		// CString::as_bytes excludes the terminating NUL and therefore matches
		// the C bridge's strlen(OPTION), including for an empty option.
		let option_length = cstring_character_length(&coption);
		let mut cunit: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, unit: &mut u8, unit_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &mut cunit[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, unit: &mut u8, errcode: &mut usize, option_length: usize, unit_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &mut cunit[0], &mut errcode, option_length, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cunit, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-SYSTEM-UNIT
	#[named]
	pub fn tqcsu(&self, option: &str, unit: &str) -> Result<(), ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let cunit:   CString = CString::new(unit)?;
		let option_length = option.len();
		let unit_length = unit.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, unit: &u8, unit_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &cunit.as_bytes()[0], unit_length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, unit: &u8, errcode: &mut usize, option_length: usize, unit_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &cunit.as_bytes()[0], &mut errcode, option_length, unit_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-INDEX-NUMBER-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqinsc(&self, name: &str) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let name_length = name.len();
		let mut indexs = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(name: &u8, name_length: usize, indexs: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], name_length, &mut indexs, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(name: &u8, indexs: &mut usize, errcode: &mut usize, name_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &mut indexs, &mut errcode, name_length);
		}
		/******************************************************************************************************/
		return wrap_result(indexs, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NAME-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqgnsc(&self, indexs: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexs: &usize, name: &mut u8, name_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexs, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexs: &usize, name: &mut u8, errcode: &mut usize, name_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexs, &mut cname[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-NAME-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqcnsc(&self, indexs: usize, name: &str) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let name_length = name.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexs: &usize, name: &u8, name_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexs, &cname.as_bytes()[0], name_length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexs: &usize, name: &u8, errcode: &mut usize, name_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexs, &cname.as_bytes()[0], &mut errcode, name_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NUMBER-OF-SYSTEM-COMPONENTS
	#[named]
	pub fn tqnosc(&self) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nscom = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(nscom: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut nscom, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(nscom: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut nscom, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(nscom, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-STOICHIOMETRY-OF-SYSTEM-COMPONENT
	#[named]
	pub fn tqstsc(&self,indexs: usize)->Result<(Vec<f64>,f64),ChemAppError>{
		let fname = func_alias(function_name!());
		let ncomp = self.tqnosc()?;
		let mut stoi : Vec<f64> = vec![0.0;ncomp];
		let mut wmass = 0.0f64;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexs: &usize, stoi: &mut f64, wmass: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexs, &mut stoi[0], &mut wmass, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexs: &usize, stoi: &mut f64, wmass: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexs, &mut stoi[0], &mut wmass, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((stoi, wmass), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
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
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(names: &u8, names_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&namememory[0], length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(names: &u8, errcode: &mut usize, names_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&namememory[0], &mut errcode, length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-INDEX-NUMBER-OF-PHASE
	#[named]
	pub fn tqinp(&self, name: &str) -> Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let cname_length = name.len();
		let mut indexp  = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cname: &u8, cname_length: usize, indexp: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &mut indexp, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cname: &u8, indexp: &mut usize, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &mut indexp, &mut errcode, cname_length);
		}
		/******************************************************************************************************/
		return wrap_result(indexp, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NAME-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgnp(&self, indexp: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, cname: &mut u8, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-MODEL-NAME-OF-PHASE
	#[named]
	pub fn tqmodl(&self, indexp: usize) -> Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, cname: &mut u8, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cname[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NUMBER-OF-PHASES
	#[named]
	pub fn tqnop(&self) -> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nphase  = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(nphase: &mut usize, errcode: &mut usize)> = self.library.get(fname.as_bytes())?;
			func(&mut nphase, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(nphase: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut nphase, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(nphase, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-INDEX-NUMBER-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqinpc(&self, indexp: usize, name: &str)-> Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut indexc  = 0;
		let mut errcode = 0;
		let cname : CString = CString::new(name)?;
		let cname_length = name.len();
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(cname: &u8, name_length: usize, indexp: &usize, indexc: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], cname_length, &indexp, &mut indexc, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cname: &u8, indexp: &usize, indexc: &mut usize, errcode: &mut usize, name_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &indexp, &mut indexc, &mut errcode, cname_length);
		}
		/******************************************************************************************************/
		return wrap_result(indexc, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NAME-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgnpc(&self, indexp: usize, indexc: usize)->Result<String, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexc: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, cname: &mut u8, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cname[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// PHASE-CONSTITUENT-IS-INCOMING-SPECIES
	#[named]
	pub fn tqpcis(&self, indexp: usize, indexc: usize)->Result<bool,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut value = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexc: &usize, value: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut value, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, value: &mut i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut value, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(value > 0, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NUMBER-OF-PHASE-CONSTITUENTS
	#[named]
	pub fn tqnopc(&self, indexp: usize)->Result<usize, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut nconst  = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, nconst: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nconst, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, nconst: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nconst, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(nconst, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-STOICHIOMETRY-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqstpc(&self, indexp: usize, indexc: usize)->Result<(Vec<f64>,f64),ChemAppError>{
		//todo!();
		let fname = func_alias(function_name!());
		let ncomp = self.tqnosc()?;
		let mut stoi : Vec<f64> = vec![0.0;ncomp];
		let mut wmass = 0.0f64;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(indexp: &usize, indexc: &usize, stoi: &mut f64, wmass: &mut f64, errcode: &mut usize)> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut stoi[0], &mut wmass, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, stoi: &mut f64, wmass: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut stoi[0], &mut wmass, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((stoi, wmass), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-CHARGE-OF-PHASE-CONSTITUENT.
	///
	/// ChemApp's `VAL` output is DOUBLE PRECISION; using `f64` prevents the
	/// native routine from writing an eight-byte value into integer storage.
	#[named]
	pub fn tqchar(&self, indexp: usize, indexc: usize)->Result<f64, ChemAppError>{
		let fname = func_alias(function_name!());
		let mut charge = 0.0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexc: &usize, charge: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut charge, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, charge: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut charge, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(charge, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-INDEX-NUMBER-OF-SUBLATTICE-CONSTITUENT
	#[named]
	pub fn tqinlc(&self, name: &str, indexp: usize, indexl: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let cname: CString = CString::new(name)?;
		let mut errcode = 0;
		let mut indexc: usize = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(name: &u8, name_len: usize, indexp: &usize, indexl: &usize, indexc: &mut usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], name.len(), &indexp, &indexl, &mut indexc, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(name: &u8, indexp: &usize, indexl: &usize, indexc: &mut usize, errcode: &mut usize, name_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cname.as_bytes()[0], &indexp, &indexl, &mut indexc, &mut errcode, name.len());
		}
		/******************************************************************************************************/
		return wrap_result(indexc, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NAME-OF-SUBLATTICE-CONSTITUENT
	#[named]
	pub fn tqgnlc(&self, indexp: usize, indexl: usize, indexc: usize)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cname : [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
		let mut errcode = 0usize;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(indexp: &usize, indexl: &usize, indexc: &usize, cname: &mut u8, cname_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexl, &indexc, &mut cname[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexl: &usize, indexc: &usize, cname: &mut u8, errcode: &mut usize, cname_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexl, &indexc, &mut cname[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NUMBER-OF-SUBLATTICES
	#[named]
	pub fn tqnosl(&self, indexp: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut nosl: usize = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, nosl: &mut usize, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nosl, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, nosl: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut nosl, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(nosl, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-NUMBER-OF-SUBLATTICE-SPECIES
	#[named]
	pub fn tqnolc(&self, indexp: usize, index: usize)->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut nosc = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, index: &usize, nosc: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &index, &mut nosc, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, index: &usize, nosc: &mut usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &index, &mut nosc, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(nosc, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-STATUS-OF-PHASE
	#[named]
	pub fn tqgsp(&self, indexp: usize)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cstatus: [u8;NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, cstatus: &mut u8, cstatus_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cstatus[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, cstatus: &mut u8, errcode: &mut usize, cstatus_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &mut cstatus[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cstatus, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-STATUS-OF-PHASE
	#[named]
	pub fn tqcsp(&self, indexp: usize, status: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cstatus: CString = CString::new(status)?;
		let cstatus_length = status.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, cstatus: &u8, cstatus_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &cstatus.as_bytes()[0], cstatus_length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, cstatus: &u8, errcode: &mut usize, cstatus_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &cstatus.as_bytes()[0], &mut errcode, cstatus_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-STATUS-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgspc(&self, indexp: usize, indexc: usize)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut cstatus: [u8;NAME_LENGTH_MAX] = [0;NAME_LENGTH_MAX];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			// TQGSPC writes the fixed-width status buffer; express that write in
			// the Rust FFI type even though pointer representation is unchanged.
			let func: Symbol<extern "system" fn(indexp: &usize, indexc: &usize, cstatus: &mut u8, cstatus_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cstatus[0], NAME_LENGTH_MAX, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, cstatus: &mut u8, errcode: &mut usize, cstatus_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &mut cstatus[0], &mut errcode, NAME_LENGTH_MAX);
		}
		/******************************************************************************************************/
		return wrap_result(fixed_fortran_string(&cstatus, NAME_LENGTH_MAX)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-STATUS-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqcspc(&self, indexp: usize, indexc: usize, status: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cstatus: CString = CString::new(status)?;
		let cstatus_length = status.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexc: &usize, status: &u8, status_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &cstatus.as_bytes()[0], cstatus_length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, status: &u8, errcode: &mut usize, status_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &cstatus.as_bytes()[0], &mut errcode, cstatus_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SET-EQUILIBRIUM-CONDITION
	#[named]
	pub fn tqsetc(&self, option: &str, indexp: usize, indexc: usize, val: f64) -> Result<i32,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut numcon  = 0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, val: &f64, numcon: &mut i32, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &val, &mut numcon, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, val: &f64, numcon: &mut i32, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &val, &mut numcon, &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result(numcon, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// REMOVE-EQUILIBRIUM-CONDITION
	#[named]
	pub fn tqremc(&self, numcon: i32) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(numcon: &i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&numcon, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(numcon: &i32, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&numcon, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SET-NAME-TEMPERATURE-PRESSURE-FOR-A-STREAM
	#[named]
	pub fn tqsttp(&self, idents: &str, vals: (f64,f64))->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let vals_ = [vals.0, vals.1];
		let cidents: CString = CString::new(idents)?;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(idents: &u8, idents_len: usize, vals: &f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &vals_[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(idents: &u8, vals: &f64, errcode: &mut usize, idents_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], &vals_[0], &mut errcode, idents.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SET-CONSTITUENT-AMOUNTS-FOR-A-STREAM
	#[named]
	pub fn tqstca(&self, idents: &str, indexp: usize, indexc: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cidents: CString = CString::new(idents)?;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(idents: &u8, idents_len: usize, indexp: &usize, indexc: &usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &indexp, &indexc, &val, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(idents: &u8, indexp: &usize, indexc: &usize, val: &f64, errcode: &mut usize, idents_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], &indexp, &indexc, &val, &mut errcode, idents.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SET-EQUILIBRIUM-CONDITION-WHEN-STREAM-INPUT
	#[named]
	pub fn tqstec(&self, option: &str, indexp: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, indexp: &usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &val, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, val: &f64, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &val, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// REMOVE-STREAM
	#[named]
	pub fn tqstrm(&self, idents: &str)->Result<(),ChemAppError>{
		//todo!();
		let fname = func_alias(function_name!());
		let cidents: CString = CString::new(idents)?;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(idents: &u8, idents_len: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(idents: &u8, errcode: &mut usize, idents_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], &mut errcode, idents.len());
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CALCULATE-EQUILIBRIUM
	#[named]
	pub fn tqce(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CALCULATE-EQUILIBRIUM-AND-LIST-RESULTS
	#[named]
	pub fn tqcel(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS
	#[named]
	pub fn tqcen(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS-AND-LIST-RESULTS
	#[named]
	pub fn tqcenl(&self, option: &str, indexp: usize, indexc: usize, vals: (f64, f64)) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let vals_ : [f64;2] = [vals.0, vals.1];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &vals_[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP
	#[named]
	pub fn tqmap(&self, option: &str, indexp: usize, indexc: usize, vals: (f64,f64))->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		let mut icont : usize = 0;
		let vals_: [f64;2] = [vals.0, vals.1];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &indexc, &vals_[0], &mut icont, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut icont, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result(icont, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP-AND-LIST-RESULTS
	#[named]
	pub fn tqmapl(&self, option: &str, indexp: usize, indexc: usize, vals: (f64,f64))->Result<usize,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut errcode = 0;
		let mut icont : usize = 0;
		let vals_: [f64;2] = [vals.0, vals.1];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &indexc, &vals_[0], &mut icont, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, vals: &f64, icont: &mut usize, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &vals_[0], &mut icont, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result(icont, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGE-LIMIT-OF-TARGET-VARIABLE
	#[named]
	pub fn tqclim(&self, option: &str, val: f64) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, val: &f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &val, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, val: &f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &val, &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// SHOW-PRESENT-SETTINGS
	#[named]
	pub fn tqshow(&self) -> Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-RESULT
	#[named]
	pub fn tqgetr(&self, option: &str, indexp: usize, indexc: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let option_length = option.len();
		let mut value = 0.0f64;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_length: usize, indexp: &usize, indexc: &usize, value: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option_length, &indexp, &indexc, &mut value, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, indexc: &usize, value: &mut f64, errcode: &mut usize, option_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &indexc, &mut value, &mut errcode, option_length);
		}
		/******************************************************************************************************/
		return wrap_result(value, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-PROPERTY-OF-A-PHASE-CONSTITUENT
	#[named]
	pub fn tqgdpc(&self, option: &str, indexp: usize, index: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let coption: CString = CString::new(option)?;
		let mut fval = 0.0f64;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(option: &u8, option_len: usize, indexp: &usize, index: &usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], option.len(), &indexp, &index, &mut fval, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(option: &u8, indexp: &usize, index: &usize, fval: &mut f64, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&coption.as_bytes()[0], &indexp, &index, &mut fval, &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result(fval, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-THERMODYNAMIC-PROPERTY-OF-A-STREAM
	#[named]
	pub fn tqstxp(&self, idents: &str, option: &str)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let cidents : CString = CString::new(idents)?;
		let coption : CString = CString::new(option)?;
		let mut errcode = 0;
		let mut fval = 0.0f64;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(idents: &u8, idents_len: usize, option: &u8, option_len: usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], idents.len(), &coption.as_bytes()[0], option.len(), &mut fval, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(idents: &u8, option: &u8, fval: &mut f64, errcode: &mut usize, idents_len: usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cidents.as_bytes()[0], &coption.as_bytes()[0], &mut fval, &mut errcode, idents.len(), option.len());
		}
		/******************************************************************************************************/
		return wrap_result(fval, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-CALCULATED-EQUILIBRIUM-SUBLATTICE-SITE-FRACTION
	#[named]
	pub fn tqgtlc(&self, indexp: usize, indexl: usize, indexc: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut fval = 0.0f64;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexl: &usize, indexc: &usize, fval: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexl, &indexc, &mut fval, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexl: &usize, indexc: &usize, fval: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexl, &indexc, &mut fval, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(fval, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-CALCULATED-QUADRUPLET-OR-PAIR-FRACTION
	#[named]
	pub fn tqbond(&self, indexp: usize, indexa: usize, indexb: usize, indexc: usize, indexd: usize)->Result<f64,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut value = 0.0;
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(indexp: &usize, indexa: &usize, indexb: &usize, indexc: &usize, indexd: &usize, value: &mut f64, errcode: &mut usize)->()>
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexa, &indexb, &indexc, &indexd, &mut value, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexa: &usize, indexb: &usize, indexc: &usize, indexd: &usize, value: &mut f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexa, &indexb, &indexc, &indexd, &mut value, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result(value, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-ERROR-MESSAGE
	#[named]
	pub fn tqerr(&self)->Result<String,ChemAppError>{
		let fname = func_alias(function_name!());
		// TQERR returns three CHARACTER*80 records. The allocation is 240
		// bytes, but the hidden length remains the length of each record: 80.
		let mut cmess : [u8; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT] = [0; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT];
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(message: &mut u8, message_len: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cmess[0], TQERR_RECORD_LENGTH, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(message: &mut u8, errcode: &mut usize, message_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&mut cmess[0], &mut errcode, TQERR_RECORD_LENGTH);
		}
		/******************************************************************************************************/
		return wrap_result(tqerr_message(&cmess)?, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-INPUT-THERMODYNAMIC-DATA-OF-PHASE-CONSTITUENT
	#[named]
	pub fn tqgdat(&self, indexp: usize, indexc: usize, option: &str, indexr: usize)->Result<Vec<f64>,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		let mut fval = [0.0;25];
		let mut nfval = 0usize;
		let coption: CString = CString::new(option)?;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(indexp: &usize, indexc: &usize, option: &u8, option_len: usize, indexr: &usize, nfval: &mut usize, fval: &mut f64, errcode: &mut usize)->()> 
				= self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &coption.as_bytes()[0], option.len(), &indexr, &mut nfval, &mut fval[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, indexc: &usize, option: &u8, indexr: &usize, nfval: &mut usize, fval: &mut f64, errcode: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &indexc, &coption.as_bytes()[0], &indexr, &mut nfval, &mut fval[0], &mut errcode, option.len());
		}
		/******************************************************************************************************/
		return wrap_result(fval[0..nfval].into_iter().copied().collect(), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// LIST-EXCESS-PARAMETERS-OF-PHASE
	#[named]
	pub fn tqlpar(&self, indexp: usize, option: &str)->Result<Vec<String>,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0usize;
		let mut nopar = 0usize;
		let coption: CString = CString::new(option)?;
		let mut lgtpar: [usize;1999] = [0usize;1999];
		let mut chrpar: [[u8;156];1999] = [[0u8;156];1999];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(indexp: &usize, option: &u8, option_len: usize, nopar: &mut usize, chrpar: &mut u8, chrpar_len: usize, lgtpar: &mut usize, noerr: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &coption.as_bytes()[0],option.len(),&mut nopar, &mut chrpar[0][0], 156, &mut lgtpar[0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, option: &u8, nopar: &mut usize, chrpar: &mut u8, lgtpar: &mut usize, noerr: &mut usize, option_len: usize, chrpar_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &coption.as_bytes()[0], &mut nopar, &mut chrpar[0][0], &mut lgtpar[0], &mut errcode, option.len(), 156);
		}
		/******************************************************************************************************/
		let vec : Vec<String> = chrpar.iter().take(nopar).map(|bytes| {String::from_utf8_lossy(bytes).trim().to_string()}).collect();
		return wrap_result(vec, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// GET-EXCESS-PARAMETERS-OF-PHASE
	#[named]
	pub fn tqgpar(&self, indexp: usize, option: &str, indexx: usize)->Result<Vec<Vec<f64>>,ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0usize;
		let coption: CString = CString::new(option)?;
		let mut noexpr = 0usize;
		let mut nvala = 0usize;
		let mut vala : [[f64;20];28] = [[0.0;20];28];
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func : Symbol<extern "system" fn(indexp: &usize, option: &u8, option_len: usize, indexx: &usize, noexpr: &mut usize, nvala: &mut usize, vala: &mut f64, noerr: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &coption.as_bytes()[0], option.len(), &indexx, &mut noexpr, &mut nvala, &mut vala[0][0], &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(indexp: &usize, option: &u8, indexx: &usize, noexpr: &mut usize, nvala: &mut usize, vala: &mut f64, noerr: &mut usize, option_len: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&indexp, &coption.as_bytes()[0], &indexx, &mut noexpr, &mut nvala, &mut vala[0][0], &mut errcode, option.len());
		}
		/******************************************************************************************************/
		let mut vecc : Vec<Vec<f64>> = Vec::new();
		for k in 0..noexpr {
			let mut vec : Vec<f64> = Vec::new();
			for m in 0..nvala {
				vec.push(vala[k][m]);
			}
			vecc.push(vec);
		}
		return wrap_result(vecc, errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// CHANGES-DATA-OF-THERMODYNAMIC-DATA-FILE
	#[named]
	pub fn tqcdat(&self, i1: usize, i2: usize, i3: usize, i4: usize, i5: usize, val: f64)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe {
			let func: Symbol<extern "system" fn(i1: &usize, i2: &usize, i3: &usize, i4: &usize, i5: &usize, val: &f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&i1, &i2, &i3, &i4, &i5, &val, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(i1: &usize, i2: &usize, i3: &usize, i4: &usize, i5: &usize, val: &f64, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&i1, &i2, &i3, &i4, &i5, &val, &mut errcode);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
	
	/*****************************************************************************************************************************************************************************************************/
	/// WRITE-DATA-FILE-IN-ASCII-FORMAT
	#[named]
	pub fn tqwasc(&self, file: &str)->Result<(),ChemAppError>{
		let fname = func_alias(function_name!());
		let cfile = CString::new(file)?;
		let cfile_length = file.len();
		let mut errcode = 0;
		/******************************************************************************************************/
		#[cfg(target_family="windows")]
		unsafe{
			let func: Symbol<extern "system" fn(cfile: &u8, cfile_length: usize, errcode: &mut usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cfile.as_bytes()[0], cfile_length, &mut errcode);
		}
		/******************************************************************************************************/
		#[cfg(target_family="unix")]
		unsafe {
			let func: Symbol<extern "C" fn(cfile: &u8, errcode: &mut usize, cfile_length: usize)->()> = self.library.get(fname.as_bytes())?;
			func(&cfile.as_bytes()[0], &mut errcode, cfile_length);
		}
		/******************************************************************************************************/
		return wrap_result((), errcode);
	}
}

/***********************************************************************************************************************************************************************************************************/
/***********************************************************************************************************************************************************************************************************/

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn fixed_output_lengths_match_the_checked_gtt_bridge() {
		assert_eq!(TQGTID_CHARACTER_LENGTH, 255);
		assert_eq!(TQGTNM_CHARACTER_LENGTH, 80);
		assert_eq!(NAME_LENGTH_MAX, 25);
		assert_eq!(TQGTRH_PROGRAM_NAME_LENGTH, 40);
		assert_eq!(TQGTRH_USER_ID_LENGTH, 255);
		assert_eq!(TQGTRH_TEXT_LENGTH, 80);
		assert_eq!(TQERR_RECORD_LENGTH, 80);
		assert_eq!(TQERR_RECORD_COUNT, 3);
	}

	#[test]
	fn fixed_fortran_string_preserves_internal_spaces_and_trims_only_padding() {
		assert_eq!(fixed_fortran_string(b"GTT - Technologies      ", 24).unwrap(), "GTT - Technologies");
		assert_eq!(fixed_fortran_string(b"Corundum                 ", 25).unwrap(), "Corundum");
		assert_eq!(fixed_fortran_string(b"    ", 4).unwrap(), "");
		assert_eq!(fixed_fortran_string(b"A B   \0not-a-record", 6).unwrap(), "A B");
		assert!(fixed_fortran_string(b"short", 6).is_err());
	}

	#[test]
	fn tqgtnm_fixed_record_preserves_complete_license_holder_text() {
		let license_holder = b"Example Research License - University";
		let mut record = [b' '; TQGTNM_CHARACTER_LENGTH];
		record[..license_holder.len()].copy_from_slice(license_holder);
		assert_eq!(
			fixed_fortran_string(&record, TQGTNM_CHARACTER_LENGTH).unwrap(),
			"Example Research License - University"
		);
	}

	#[test]
	fn tqerr_is_three_fixed_width_records() {
		let mut buffer = [b' '; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT];
		buffer[..15].copy_from_slice(b"Example Program");
		let second = b"Copyright Example Organization, 100 Research Road, Example City";
		buffer[TQERR_RECORD_LENGTH..TQERR_RECORD_LENGTH + second.len()].copy_from_slice(second);
		let third = b"https://example.invalid";
		buffer[2 * TQERR_RECORD_LENGTH..2 * TQERR_RECORD_LENGTH + third.len()].copy_from_slice(third);
		assert_eq!(
			tqerr_message(&buffer).unwrap(),
			"Example Program\nCopyright Example Organization, 100 Research Road, Example City\nhttps://example.invalid"
		);
	}

	#[test]
	fn tqgsu_input_length_excludes_only_the_cstring_nul() {
		assert_eq!(cstring_character_length(&CString::new("").unwrap()), 0);
		assert_eq!(cstring_character_length(&CString::new("Pressure").unwrap()), 8);
	}

	#[test]
	fn tqchar_exposes_the_native_double_precision_result() {
		let _: fn(&Engine, usize, usize) -> Result<f64, ChemAppError> = Engine::tqchar;
	}
}
