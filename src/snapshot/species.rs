// chemapp_rs::snapshot::species.rs

#[derive(Debug,Clone)]
pub struct SpeciesSnapshot {
	pub indexp : usize,
	pub indexl : usize,
	pub indexs : usize,
	pub name   : String,
	pub x      : f64,
}