// Calculator state snapshot

pub mod system;
pub mod component;
pub mod phase;
pub mod constituent;
pub mod species;
pub mod bond;

use crate::calculator::Calculator;
pub use system::SystemSnapshot;
pub use component::SystemComponentSnapshot;
pub use phase::PhaseSnapshot;
pub use constituent::ConstituentSnapshot;
pub use species::SpeciesSnapshot;
pub use bond::BondSnapshot;

/// A snapshot of a calculator state
#[derive(Debug,Clone)]
pub struct CalculatorSnapshot {
	system     : SystemSnapshot,
	components : Vec<SystemComponentSnapshot>,
	phases     : Vec<PhaseSnapshot>,
}

impl CalculatorSnapshot {
	
	pub fn new(calculator: &Calculator)->Self {
		let system = calculator.system().snapshot();
		let components : Vec<SystemComponentSnapshot> = calculator.components().map(|c| c.snapshot()).collect();
		let phases : Vec<PhaseSnapshot> = calculator.phases().map(|p| p.snapshot()).collect();
		return Self {
			system,
			components,
			phases,
		};
	}
	
}