extern crate libloading;

use libloading::{Library};
//use serde::{Serialize,Serializer};

use chemformula::{Transform};
pub use crate::calculator::{Calculator, ComponentIterator, PhaseIterator, ConstituentIterator};

pub mod error;
pub mod defs;
pub mod native;
pub mod calculator;

static DEFAULT_LIBNAME : &str = r"ca_vc_e_local.dll";

//#[derive(Serialize, Deserialize)]
pub struct Engine{
	pub n_isothermal: usize,
	pub n_target: usize,
	library_name: String,
	library: Library,
}

impl Default for Engine {
	fn default()->Engine {
		return Engine::new(DEFAULT_LIBNAME).unwrap();
	}
}

//#[derive(Serialize, Deserialize)]
pub struct SystemDimensions{
	nconstituents: i32,       // na
	ncomponents: i32,         // nb
	nmixtures: i32,           // nc
	nexcess_gibbs: i32,       // nd
	nexcess_magnetic: i32,    // ne
	nsublattices: i32,        // nf
	nspecies: i32,            // ng
	nconstituents_mqm: i32,   // nh
	nranges_constituent: i32, // ni
	nranges: i32,             // nj
	ndependent: i32,          // nk
}

impl SystemDimensions {
	pub fn new()->SystemDimensions{
		SystemDimensions{
			nconstituents:       0,
			ncomponents:         0,
			nmixtures:           0,
			nexcess_gibbs:       0,
			nexcess_magnetic:    0,
			nsublattices:        0,
			nspecies:            0,
			nconstituents_mqm:   0,
			nranges_constituent: 0,
			nranges:             0,
			ndependent:          0,
		}
	}
}

impl<T> ComponentIterator for T where T : Iterator<Item=usize>{}

impl<T> PhaseIterator for T where T : Iterator<Item=usize>{}

impl<T> ConstituentIterator for T where T : Iterator<Item=(usize,usize)>{}