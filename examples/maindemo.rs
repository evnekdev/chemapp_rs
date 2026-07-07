// chemapp_rs//examples/maindemo.rs
use chemapp_rs::{Engine,ChemAppError};

pub fn main(){
	/**********************************************************************************************************************/
	#[cfg(all(target_family = "windows", target_pointer_width = "32"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_local.dll";
	#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_x64.dll";
	/**********************************************************************************************************************/
	let datafile_dat = r"c:\_WORK\Code\Rust\workspace\chemapp_rs\data\cosi.dat";
	/**********************************************************************************************************************/
	let engine = Engine::new(libname).unwrap();
	/**********************************************************************************************************************/
	// Initialize the library
	let _    = engine.tqini().unwrap();
	/**********************************************************************************************************************/
	// Print the copyright message
	//let _    = engine.tqcprt().unwrap();
	//let cprt = engine.tqerr().unwrap();
	/**********************************************************************************************************************/
	// ChemApp library version
	let vers = engine.tqvers().unwrap();
	println!("ChemApp version = {:?}", &vers);
	/**********************************************************************************************************************/
	// Internal array sizes
	let sizes = engine.tqsize().unwrap();
	println!("Internal array sizes:\n{:?}", &sizes);
	/**********************************************************************************************************************/
	// default FORTRAN unit for tqrfil
	let unitno = engine.tqgio("FILE").unwrap();
	println!("The thermochemical data will be read from the file associated with unit {:?}", &unitno);
	/**********************************************************************************************************************/
	let _ = engine.tqopna(datafile_dat, unitno).unwrap();
	let _ = engine.tqrfil().unwrap();
	let _ = engine.tqclos(unitno).unwrap();
	/**********************************************************************************************************************/
	// Used array dimensions
	let used = engine.tqused().unwrap();
	println!("Used array sizes:\n{:?}", &used);
	/**********************************************************************************************************************/
	/**********************************************************************************************************************/
	// get system units
	let punit = engine.tqgsu("Pressure").unwrap();
	let vunit = engine.tqgsu("Volume").unwrap();
	let tunit = engine.tqgsu("Temperature").unwrap();
	let eunit = engine.tqgsu("Energy").unwrap();
	let aunit = engine.tqgsu("Amount").unwrap();
	println!("Pressure unit: {:?}", &punit);
	println!("Volume unit: {:?}", &vunit);
	println!("Temperature unit: {:?}", &tunit);
	println!("Energy unit: {:?}", &eunit);
	println!("Amount unit: {:?}", &aunit);
	/**********************************************************************************************************************/
	// change "Amount" unit to grams
	let _ = engine.tqcsu("Amount", "gram").unwrap();
	/**********************************************************************************************************************/
	/**********************************************************************************************************************/
	let nscom = engine.tqnosc().unwrap();
	println!("Number of system components : {:?}", &nscom);
	/**********************************************************************************************************************/
	let name = engine.tqgnsc(1).unwrap();
	let (stoic, wmass) = engine.tqstsc(1).unwrap();
	println!("System component {:?}, stoic = {:?}, wmass = {:?}", &name, &stoic, &wmass);
	let index = engine.tqinsc(&name).unwrap();
	println!("Index number of {:?} is {:?}", &name, &index);
	/**********************************************************************************************************************/
	let newsyscomp = vec!["SiO", "SiC", "CO"];
	let _ = engine.tqcsc(&newsyscomp).unwrap();
	println!("System components changed to {:?}", &newsyscomp);
	for k in 1..=nscom {
		let name = engine.tqgnsc(k).unwrap();
		let (stoic, wmass) = engine.tqstsc(k).unwrap();
		println!("Name of new system component {:?}: {:?}, stoic = {:?}, wmass = {:?}", &k, &name, &stoic, &wmass);
	}
	/**********************************************************************************************************************/
	/**********************************************************************************************************************/
}