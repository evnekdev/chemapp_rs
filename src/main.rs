use chemapp::{native::*, calculator::*, *};
use nalgebra::{dvector};


pub fn main(){
	println!("chemapp module");
	/*
	let engine = Engine::default();
	engine.tqini();
	engine.load_datafile(r"c:\_WORK\Continuous\diagrams\007_Al2O3-SiO2\MS15_Al-Si-O.dat");
	*/
	let mut calculator = Calculator::from_library(r"ca_vc_e_local.dll", r"c:\_WORK\Continuous\diagrams\007_Al2O3-SiO2\BETA_Al-Si-O.dat").unwrap();
	calculator.set_transform(&["SiO2","Al2O3"]);
	calculator.calculate_isothermal_d(&dvector![0.1, 0.9], 2200.0);
	for idx in (0..100).components_valid(&calculator).components_names(&calculator) {
		println!("idx = {:?}", &idx);
	}
	for idx in (0..100).phases_valid(&calculator).phases_status_eliminated(&calculator).phases_names(&calculator) {
		println!("idx = {:?}", &idx);
	}
	
	for idx in (0..100).phases_valid(&calculator).phases_constituents(&calculator).constituents_HM(&calculator){
		println!("idx = {:?}", &idx);
	}
}