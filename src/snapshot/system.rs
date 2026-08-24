use crate::entities::system::System;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
/// Engine-independent copy of whole-system calculated properties.
pub struct SystemSnapshot {
    /// Temperature (`T`).
    pub t: f64,
    /// Pressure (`P`).
    pub p: f64,
    /// Total volume (`VT`).
    pub vt: f64,
    /// Total amount (`A`).
    pub a: f64,
    /// Heat capacity (`CP`).
    pub cp: f64,
    /// Enthalpy (`H`).
    pub h: f64,
    /// Entropy (`S`).
    pub s: f64,
    /// Gibbs energy (`G`).
    pub g: f64,
    /// Volume (`V`).
    pub v: f64,
}

impl SystemSnapshot {
    /// Captures all exposed values from the live system view.
    pub fn new(system: &System<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            t: system.t()?,
            p: system.p()?,
            vt: system.vt()?,
            a: system.a()?,
            cp: system.cp()?,
            h: system.h()?,
            s: system.s()?,
            g: system.g()?,
            v: system.v()?,
        })
    }
}
