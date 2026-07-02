// species.rs

use crate::calculator::Calculator;

pub struct SpeciesIterator<'a> {
	calculator : &'a Calculator,
}