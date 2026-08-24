// chemapp_rs::iterator.rs
//! Useful iterators over system components, phases, and phase constituents in a datafile.

/// Model-aware TQBOND pair/quadruplet iteration.
pub mod bond;
/// System-component iteration.
pub mod component;
/// Ordinary phase-constituent iteration.
pub mod constituent;
/// Phase iteration.
pub mod phase;
/// Sublattice-species iteration.
pub mod species;

pub use bond::BondIterator;
pub use component::SystemComponentIterator;
pub use constituent::ConstituentIterator;
pub use phase::PhaseIterator;
pub use species::SpeciesIterator;
