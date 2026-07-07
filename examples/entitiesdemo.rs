// entitiesdemo.rs
use chemapp_rs::{Calculator,Engine,ChemAppError};

pub fn main(){
	/**********************************************************************************************************************/
	#[cfg(all(target_family = "windows", target_pointer_width = "32"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_local.dll";
	#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_x64.dll";
	/**********************************************************************************************************************/
	let datafile_dat = r"c:\_WORK\Code\Rust\workspace\chemapp_rs\data\cosi.dat";
	/**********************************************************************************************************************/
	let calculator = Calculator::from_library(libname, datafile_dat).unwrap();
	/**********************************************************************************************************************/
	let _ = calculator.engine.tqsetc("T",  0, 0, 1200.0).unwrap();
	let _ = calculator.engine.tqsetc("P",  0, 0, 1.0).unwrap();
	let _ = calculator.engine.tqsetc("IA", 0, 1, 1.0).unwrap();
	let _ = calculator.engine.tqsetc("IA", 0, 2, 0.2).unwrap();
	let _ = calculator.engine.tqsetc("IA", 0, 3, 1.5).unwrap();
	//let _ = calculator.engine.tqshow().unwrap();
	let _ = calculator.engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
	/**********************************************************************************************************************/
	calculator.print_system();
	/**********************************************************************************************************************/
	calculator.print_components();
	/**********************************************************************************************************************/
}