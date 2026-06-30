// interactions_magnetic.rs

use chemapp_rs::{Engine};

pub fn main(){
	/**********************************************************************************************************************/
	#[cfg(all(target_family = "windows", target_pointer_width = "32"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_local.dll";
	#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
	let libname = r"c:\_WORK\Code\ca_vc_e_x64.dll";
	/**********************************************************************************************************************/
	let datafile_dat = r"c:\_WORK\Code\Rust\workspace\chemapp_rs\data\EN22_Ca-Fe-Si-O.DAT";
	/**********************************************************************************************************************/
	let engine = Engine::new(libname).unwrap();
	/**********************************************************************************************************************/
	// Initialize the library
	let _    = engine.tqini().unwrap();
	/**********************************************************************************************************************/
	let unitno = engine.tqgio("FILE").unwrap();
	/**********************************************************************************************************************/
	let _ = engine.tqopna(datafile_dat, unitno).unwrap();
	let _ = engine.tqrfil().unwrap();
	let _ = engine.tqclos(unitno).unwrap();
	/**********************************************************************************************************************/
	let nphases = engine.tqnop().unwrap();
	for p in 1..nphases+1 {
		let model = engine.tqmodl(p).unwrap();
		let name  = engine.tqgnp(p).unwrap();
		if model != "PURE" {
			println!("MAGN INTERACTIONS IN {:?}", &name);
			if let Ok(interactions) = engine.tqlpar(p, "M"){
				println!("{}", &interactions.join("\n"));
			}
		}
	}
	/**********************************************************************************************************************/
}