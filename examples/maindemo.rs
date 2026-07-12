// chemapp_rs//examples/maindemo.rs
use std::path::PathBuf;
use chemapp_rs::{Engine,ChemAppError};

pub fn main(){
	/**********************************************************************************************************************/
	let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	#[cfg(all(target_family = "windows", target_pointer_width = "32"))]
	let libpath = project_dir.join("windows").join("ca_vc_e_local.dll");
	#[cfg(all(target_family = "windows", target_pointer_width = "64"))]
	let libpath = project_dir.join("windows").join("ca_vc_e_x64.dll");
	#[cfg(target_family = "unix")]
	let libpath = project_dir.join("linux").join("libLChemAppS.so");
	/**********************************************************************************************************************/
	let datafile_path = project_dir.join("data").join("cosi.dat");
	let libname = libpath.to_str().unwrap();
	let datafile_dat = datafile_path.to_str().unwrap();
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
	// Continue cademo1.c immediately after its first tqcsc section.
	let (stoic, wmass) = engine.tqstsc(1).unwrap();
	println!("Updated component 1: stoic = {:?}, wmass = {:?}", stoic, wmass);
	let nphase = engine.tqnop().unwrap();
	let phase_name = engine.tqgnp(1).unwrap();
	println!("Phases: {:?}; phase 1: {:?}; reverse lookup: {:?}", nphase, phase_name, engine.tqinp(&phase_name).unwrap());
	println!("Models: phase 1 = {:?}, last phase = {:?}", engine.tqmodl(1).unwrap(), engine.tqmodl(nphase).unwrap());
	let npcgas = engine.tqnopc(1).unwrap();
	let constituent_name = engine.tqgnpc(1, 1).unwrap();
	println!("Phase 1 constituents: {:?}; first: {:?}; reverse lookup: {:?}", npcgas, constituent_name, engine.tqinpc(1, &constituent_name).unwrap());
	println!("Incoming species permitted: {:?}", engine.tqpcis(1, 1).unwrap());
	let (stoic, wmass) = engine.tqstpc(1, 1).unwrap();
	println!("Phase constituent stoic = {:?}, wmass = {:?}", stoic, wmass);

	engine.tqcsc(&["C", "O", "Si"]).unwrap();
	engine.tqcsp(1, "eliminated").unwrap();
	println!("Phase status: {:?}", engine.tqgsp(1).unwrap());
	engine.tqcsp(1, "entered").unwrap();
	println!("Phase status: {:?}", engine.tqgsp(1).unwrap());
	engine.tqcspc(1, 1, "dormant").unwrap();
	println!("Constituent status: {:?}", engine.tqgspc(1, 1).unwrap());
	engine.tqcspc(1, 1, "entered").unwrap();
	println!("Constituent status: {:?}", engine.tqgspc(1, 1).unwrap());

	let _ = engine.tqsetc("ia ", 1, 4, 1.0).unwrap();
	engine.tqcsu("Amount", "mol").unwrap();
	let _ = engine.tqsetc("ia ", 1, 12, 3.0).unwrap();
	let numcon = engine.tqsetc("ia ", 1, 8, 2.0).unwrap();
	engine.tqremc(numcon).unwrap();
	let _ = engine.tqsetc("t ", 0, 0, 1800.0).unwrap();
	engine.tqclim("plow", 1e-49).unwrap();
	engine.tqshow().unwrap();
	engine.tqce(" ", 0, 0, (0.0, 0.0)).unwrap();
	engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
	let _ = engine.tqsetc("t ", 0, 0, 1850.0).unwrap();
	engine.tqcen(" ", 0, 0, (0.0, 0.0)).unwrap();
	let _ = engine.tqsetc("t ", 0, 0, 1900.0).unwrap();
	engine.tqcenl(" ", 0, 0, (0.0, 0.0)).unwrap();

	let result_path = project_dir.join("result");
	engine.tqcio("LIST", 21).unwrap();
	engine.tqopen(result_path.to_str().unwrap(), 21).unwrap();
	engine.tqwstr("LIST", "Output from tqcel (ChemSage result table):").unwrap();
	engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
	engine.tqclos(21).unwrap();
	engine.tqcio("LIST", 6).unwrap();

	let component_name = engine.tqgnsc(1).unwrap();
	println!("Mole fraction of {:?}: {:?}", component_name, engine.tqgetr("xp ", 1, 1).unwrap());
	println!("Equilibrium amount: {:?}; phase activity: {:?}", engine.tqgetr("a ", 1, 1).unwrap(), engine.tqgetr("ac ", 1, 0).unwrap());
	for indexc in 1..=npcgas {
		println!("Fugacity of {:?}: {:?}", engine.tqgnpc(1, indexc).unwrap(), engine.tqgetr("ac", 1, indexc).unwrap());
	}
	println!("Dimensionless G: {:?}", engine.tqgdpc("G", 1, 1).unwrap());

	if engine.tqlite().unwrap() {
		println!("Target calculations omitted for the ChemApp light version.");
	} else {
		let liquid = engine.tqinp("SiO2(liq").unwrap();
		let _ = engine.tqsetc("a", liquid, 0, 0.0).unwrap();
		engine.tqcel("t", 0, 0, (2000.0, 0.0)).unwrap();
		println!("Formation temperature: {:?}", engine.tqgetr("t", 0, 0).unwrap());
		engine.tqremc(-2).unwrap();
		let quartz = engine.tqinp("SiO2(quartz)").unwrap();
		let _ = engine.tqsetc("IA", quartz, 0, 1.0).unwrap();
		let interval = (300.0, 3000.0);
		let mut more = engine.tqmap("tf", 0, 0, interval).unwrap();
		let mut result_number = 1;
		println!("Mapping result: {:?} K", engine.tqgetr("t", 0, 0).unwrap());
		while more != 0 {
			more = if result_number == 2 { engine.tqmapl("tn", 0, 0, interval).unwrap() } else { engine.tqmap("tn", 0, 0, interval).unwrap() };
			result_number += 1;
			println!("Mapping result: {:?} K", engine.tqgetr("t", 0, 0).unwrap());
		}
	}

	engine.tqremc(-2).unwrap();
	for stream in ["stream1", "stream2", "stream3"] { engine.tqsttp(stream, (1000.0, 1.0)).unwrap(); }
	engine.tqstca("stream1", 1, 4, 1.0).unwrap();
	engine.tqstca("stream2", 1, 12, 3.0).unwrap();
	engine.tqstca("stream3", 1, 8, 2.0).unwrap();
	engine.tqstrm("stream3").unwrap();
	engine.tqstec("t ", 0, 1800.0).unwrap();
	engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
	println!("Enthalpy of stream1: {:?}", engine.tqstxp("stream1", "H").unwrap());

	let sublattice_file = project_dir.join("data").join("subl-ex.dat");
	if sublattice_file.exists() {
	engine.tqopna(sublattice_file.to_str().unwrap(), unitno).unwrap();
	engine.tqrfil().unwrap();
	engine.tqclos(unitno).unwrap();
	let sigma = engine.tqinp("SIGMA:30#1").unwrap();
	for indexl in 1..=engine.tqnosl(sigma).unwrap() {
		for indexc in 1..=engine.tqnolc(sigma, indexl).unwrap() {
			let name = engine.tqgnlc(sigma, indexl, indexc).unwrap();
			println!("Sublattice {:?}, constituent {:?}: {:?}", indexl, engine.tqinlc(&name, sigma, indexl).unwrap(), name);
		}
	}
	let _ = engine.tqsetc("T", 0, 0, 1000.0).unwrap();
	for (name, amount) in [("Co", 0.25), ("Cr", 0.25), ("Fe", 0.50)] {
		let indexc = engine.tqinsc(name).unwrap();
		let _ = engine.tqsetc("ia", 0, indexc, amount).unwrap();
	}
	engine.tqce(" ", 0, 0, (0.0, 0.0)).unwrap();
	for indexp in 1..=engine.tqnop().unwrap() {
		if engine.tqgetr("a", indexp, 0).unwrap() > 0.0 && engine.tqmodl(indexp).unwrap().starts_with("SUB") {
			println!("Sublattice fractions in {:?}", engine.tqgnp(indexp).unwrap());
			for indexl in 1..=engine.tqnosl(indexp).unwrap() {
				for indexc in 1..=engine.tqnolc(indexp, indexl).unwrap() {
					println!("{:?}: {:?}", engine.tqgnlc(indexp, indexl, indexc).unwrap(), engine.tqgtlc(indexp, indexl, indexc).unwrap());
				}
			}
		}
	}
	} else {
		println!("Skipping sublattice functions because subl-ex.dat is unavailable.");
	}

	println!("Licensee user ID: {:?}", engine.tqgtid().unwrap());
	println!("Licensee name: {:?}; program ID: {:?}", engine.tqgtnm().unwrap(), engine.tqgtpi().unwrap());
	let (dongle_type, dongle_id) = engine.tqgthi().unwrap();
	let (expiry_month, expiry_year) = engine.tqgted().unwrap();
	println!("Dongle: {:?} {:?}; expiry: {:?}/{:?}", dongle_type, dongle_id, expiry_month, expiry_year);
	let error_unit = engine.tqgio("ERROR").unwrap();
	engine.tqcio("ERROR", 0).unwrap();
	let transparent_file = project_dir.join("data").join("cosiex.cst");
	let transparent_opened = engine.tqopnt(transparent_file.to_str().unwrap(), unitno).is_ok();
	engine.tqcio("ERROR", error_unit).unwrap();
	if transparent_opened {
		engine.tqrcst().unwrap();
		engine.tqclos(unitno).unwrap();
		println!("Transparent file header: {:#?}", engine.tqgtrh().unwrap());
	} else {
		println!("Skipping transparent file functions because cosiex.cst is unavailable.");
	}
	println!("End of output translated from cademo1.");
}
