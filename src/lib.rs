#![allow(unused_variables)]

//! This crate is an unofficial port of ChemApp Library for thermochemical calculations (GTT Technologies at <https://gtt-technologies.de/software/chemapp/>).
//!    
//! From the official site:
//! "ChemApp provides the powerful calculation capabilities of FactSage in the form of a programmer’s library. It consists of a rich set of subroutines which provides all the necessary tools for the calculation of complex multicomponent, multiphase chemical equilibria and the determination of the associated energy balances."
//!   
//! ChemApp library was originally written in Fortran and is distributed as a dll / loadable dynamic library; to run the full version, you need a license (please contact GTT for more information).
//!   
//! Officially, ChemApp has programming support in the following languages : C/C++, Delphi, Fortran and Visual Basic. Recently, official support for Matlab and Python has been added.
//!   
//! The current crate is not part of the official suite; however, it has been thouroughly tested and used in several proprietary applications.
//! Rust language offers the benefits of low-level languages as as C in its execution speed as well as a rich ecosystem of mature third-party libraries (crates) regarding many programming aspects, such as numerical computation, GUI development, cross-platform support, etc.
//!   
//! Almost every mature programming language offers so called FFI (Foreign Function Interface). Modern software development is not limited to the choice of one language; with the necessary skills, components written in different languages can be combined together. For example, after compilation, Fortran code (such as ChemApp or other scientific numerical computation software) exposes so-called ABI (Application Binary Interface) in form of exported functions in a DLL.
//!   
//!A DLL can be linked to a hosting process using either *static* and *dynamic* dispatch mechanism. Static dispatch bakes in the name of the DLL into the application during compilation (C, Delphi), to do that, *.lib files are used (these files contain DLL function stubs which are later found when the process is loaded into the memory).
//! On the contrary, dynamic dispatch delays locating a DLL until a LoadLibrary function from Windows SDK is explicitly called inside the code. This allows more flexibility, for example, one can pass a dll name (even different versions) as a function parameter in the application; the user has flexibility to switch between different versions of the same DLL without recompilation (nevertheless, the DLL and EXE bitnesses have to match, operating systems do not support loading 32bit DLLs in 64bit applications and otherwise).
//!   
//! Dynamic dispatch also offers a way to parallelize a large number of similar calculations. When a shared library is loaded into different applications, the operating system usually keeps only a single copy of its code section for efficiency, since code sections are always read-only. However, for each application, the operating system keeps a separate copy of the library variables (everything that is mutable and changes during the calculations). On Linux, it is possible to load the same library into the same process several times distinctly; Windows officially does not support that, but the author discovered that if a DLL is manually copied with renaming, two idential DLLs CAN be loaded at the same time separately. This allows to split the computational load between different copies of ChemApp.

extern crate libloading;

use libloading::{Library};

pub use crate::calculator::{Calculator, ComponentIterator, PhaseIterator, ConstituentIterator};

pub mod error;
pub mod defs;
pub mod native;
pub mod calculator;
pub mod interactions;

static DEFAULT_LIBNAME : &str = r"ca_vc_e_local.dll";

/// An encapsulation of a single loaded DLL - different instances correspond to different DLLs. ChemApp tq... functions are exported as methods, rather than independent functions to support multiple DLL loading.
#[derive(Debug)]
pub struct Engine {
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

/// An abstraction over system info returned by `tqused` and `tqsize` functions.
#[derive(Debug)]
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