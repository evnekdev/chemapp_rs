//! Unofficial Rust bindings and high-level workflows for the proprietary
//! [ChemApp](https://gtt-technologies.de/software/chemapp/) thermochemical
//! equilibrium library.
//!
//! ChemApp performs the thermodynamic calculations; this crate dynamically
//! loads a separately obtained ChemApp DLL or shared library. No ChemApp
//! binary, licence, or commercial thermodynamic database is distributed in
//! the crates.io package.
//!
//! Most applications should start with [`Calculator`]. [`Engine`] exposes the
//! lower-level, native-oriented `TQ...` interface when exact ChemApp control is
//! required. Live entities reflect the current native state, while
//! [`CalculatorSnapshot`] owns a result that remains usable after later native
//! calls change that state.
//!
//! # Loading and inspecting a system
//!
//! ```no_run
//! use chemapp_rs::{Calculator, ChemAppError};
//!
//! fn main() -> Result<(), ChemAppError> {
//!     let calculator = Calculator::from_library(
//!         "path/to/chemapp/library",
//!         "path/to/system.dat",
//!     )?;
//!     println!("ChemApp version: {}", calculator.engine.tqvers()?);
//!     for phase in calculator.phases()? {
//!         println!("{}: {}", phase.name()?, phase.model()?);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ChemApp and the Rust process must have matching architectures. Native
//! execution may also require a valid ChemApp licence. See the repository
//! README for installation, examples, platform evidence, and troubleshooting.

extern crate libloading;

pub use crate::calculator::Calculator;
pub use crate::entities::bond::{Bond, BondKind};
pub use crate::entities::species::{Species, SpeciesRef};
pub use crate::error::ChemAppError;
pub use crate::interactions::{
    Interaction, InteractionChannel, InteractionCrossCheck, InteractionCrossCheckRequest,
    InteractionDescriptor, InteractionDescriptorCrossCheck, InteractionDescriptorRecovery,
    InteractionDescriptorSource, InteractionMember, InteractionMutationSupport, InteractionOrder,
    InteractionParameter, InteractionParameterAddress, InteractionParameterCell,
    InteractionParameterRole, InteractionRaw, InteractionRecoveryReason,
    InteractionRecoveryRequest, InteractionResolution, MagneticInteractionRole,
    NativeInteractionIndex, NativePoweredMember, PhaseInteractionReport,
    ResolvedInteractionDescriptor, ResolvedPoweredMember,
};
pub use crate::iterator::{
    BondIterator, ConstituentIterator, PhaseIterator, SpeciesIterator, SystemComponentIterator,
};
pub use crate::native::Engine;
pub use crate::snapshot::{CalculatorSnapshot, SnapshotOptions, StreamSnapshot};
use std::fmt;

mod abi;
pub mod cache;
pub mod calculator;
pub mod defs;
pub mod entities;
pub mod error;
pub mod interactions;
pub mod iterator;
pub mod native;
pub mod parse;
pub mod snapshot;
mod table;

/// The eleven dimensions returned by [`Engine::tqsize`] and [`Engine::tqused`].
///
/// `TQSIZE` reports compiled capacities; `TQUSED` reports the corresponding
/// maxima required by the currently loaded system.
#[derive(Clone, Default)]
pub struct SystemDimensions {
    /// `NA`: total phase constituents.
    pub constituents: i32,
    /// `NB`: system components.
    pub system_components: i32,
    /// `NC`: mixture phases.
    pub mixture_phases: i32,
    /// `ND`: excess Gibbs-energy coefficients for one mixture phase.
    pub excess_gibbs_coefficients_per_phase: i32,
    /// `NE`: excess magnetic coefficients for one mixture phase.
    pub excess_magnetic_coefficients_per_phase: i32,
    /// `NF`: sublattices for one mixture phase.
    pub sublattices_per_phase: i32,
    /// `NG`: constituents on one sublattice.
    pub constituents_per_sublattice: i32,
    /// `NH`: oxide constituents in one Gaye–Kapoor–Frohberg or modified
    /// quasichemical phase.
    pub gkf_mqm_oxide_constituents_per_phase: i32,
    /// `NI`: Gibbs-energy/heat-capacity equations for one constituent and the
    /// native leading dimension of the `TQGPAR` value array.
    pub equations_per_constituent: i32,
    /// `NJ`: total Gibbs-energy/heat-capacity equations.
    pub equations: i32,
    /// `NK`: constituents with pressure- and temperature-dependent molar volume.
    pub pt_dependent_volume_constituents: i32,
}

impl SystemDimensions {
    /// Returns an all-zero dimension set for FFI output initialization.
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Debug for SystemDimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SystemDimensions:")?;
        writeln!(f, "  {:<44} {:?}", "Constituents (NA)", self.constituents)?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "System components (NB)", self.system_components
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "Mixture phases (NC)", self.mixture_phases
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "Excess Gibbs coefficients / phase (ND)", self.excess_gibbs_coefficients_per_phase
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "Excess magnetic coefficients / phase (NE)",
            self.excess_magnetic_coefficients_per_phase
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "Sublattices / phase (NF)", self.sublattices_per_phase
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "Constituents / sublattice (NG)", self.constituents_per_sublattice
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "GKF/MQM oxide constituents / phase (NH)", self.gkf_mqm_oxide_constituents_per_phase
        )?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "G/CP equations / constituent (NI)", self.equations_per_constituent
        )?;
        writeln!(f, "  {:<44} {:?}", "G/CP equations (NJ)", self.equations)?;
        writeln!(
            f,
            "  {:<44} {:?}",
            "P,T-dependent volume constituents (NK)", self.pt_dependent_volume_constituents
        )?;
        Ok(())
    }
}

/*****************************************************************************************************************************************************************************************************/
/*****************************************************************************************************************************************************************************************************/

/// Licensing and provenance metadata returned from a transparent `.cst` file.
#[derive(Clone)]
pub struct TransparentHeader {
    /// Transparent-file format version.
    pub version: i32,
    /// Program that wrote the file.
    pub name_writing_program: String,
    /// Three-part version of the writing program.
    pub version_writing_program: [i32; 3],
    /// Program permitted to read the file.
    pub name_reading_program: String,
    /// Minimum permitted three-part reading-program version.
    pub minversion_reading_program: [i32; 3],
    /// File creation date/time fields returned by ChemApp.
    pub creation_date: [i32; 6],
    /// File expiry date/time fields returned by ChemApp.
    pub expiry_date: [i32; 6],
    /// Allowed ChemApp user identifiers.
    pub user_ids_allowed: String,
    /// Allowed ChemApp licence-holder names.
    pub license_holders_allowed: String,
    /// File remark text.
    pub remark: String,
}

impl fmt::Debug for TransparentHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TransparentHeader:")?;
        writeln!(f, "  {:<30} {:?}", "Version", &self.version)?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Writing program name", &self.name_writing_program
        )?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Writing program version", &self.version_writing_program
        )?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Reading program name", &self.name_reading_program
        )?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Reading program min version", &self.minversion_reading_program
        )?;
        writeln!(f, "  {:<30} {:?}", "Creation date", &self.creation_date)?;
        writeln!(f, "  {:<30} {:?}", "Expiration date", &self.expiry_date)?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Allowed user ids", &self.user_ids_allowed
        )?;
        writeln!(
            f,
            "  {:<30} {:?}",
            "Allowed license holders", &self.license_holders_allowed
        )?;
        writeln!(f, "  {:<30} {:?}", "Remark", &self.remark)?;
        Ok(())
    }
}

/*****************************************************************************************************************************************************************************************************/
/*****************************************************************************************************************************************************************************************************/
