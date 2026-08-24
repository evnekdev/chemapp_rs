//! Compatibility re-exports for the typed interaction parser.
//!
//! The former parser mixed syntax parsing, native name lookup, panicking
//! `unwrap` calls, and inconsistent two-sublattice offsets. The authoritative
//! implementation now separates raw, parsed, and resolved representations.

pub use crate::interactions::{parse_interaction_descriptor, InteractionDescriptor};
