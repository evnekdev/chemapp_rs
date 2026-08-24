use crate::entities::system::System;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    pub t: f64,
    pub p: f64,
    pub vt: f64,
    pub a: f64,
    pub cp: f64,
    pub h: f64,
    pub s: f64,
    pub g: f64,
    pub v: f64,
}

impl SystemSnapshot {
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
