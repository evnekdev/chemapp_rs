// Calculator state snapshot

pub mod system;
pub mod component;
pub mod phase;
pub mod constituent;

use crate::calculator::Calculator;
pub use system::SystemSnapshot;
pub use component::SystemComponentSnapshot;
pub use phase::PhaseSnapshot;
pub use constituent::ConstituentSnapshot;

/// A snapshot of a calculator state
#[derive(Debug,Clone)]
pub struct CalculatorSnapshot {
	system : SystemSnapshot,
	components : Vec<SystemComponentSnapshot>,
	phases : Vec<PhaseSnapshot>,
}

impl CalculatorSnapshot {
	
	pub fn new(calculator: &Calculator)->Self {
		todo!();
	}
	
}