// bond.rs

use crate::calculator::Calculator;

pub struct BondIterator<'a> {
	calculator : &'a Calculator,
}