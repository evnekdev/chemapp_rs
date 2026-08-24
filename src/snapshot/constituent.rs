use crate::entities::constituent::Constituent;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
pub struct ConstituentSnapshot {
    pub phase_index: usize,
    pub phase_name: String,
    pub index: usize,
    pub status: String,
    pub name: String,
    pub charge: f64,
    pub wmass: f64,
    pub stoic: Vec<f64>,
    pub incoming_allowed: bool,
    pub ia: f64,
    pub a: f64,
    pub ac: f64,
    pub mu: f64,
    pub h: f64,
    pub s: f64,
    pub g: f64,
    pub cp: f64,
    pub v: f64,
    pub hm: f64,
    pub sm: f64,
    pub gm: f64,
    pub cpm: f64,
    pub vm: f64,
}

impl ConstituentSnapshot {
    pub fn new(constituent: &Constituent<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            phase_index: constituent.phase_index(),
            phase_name: constituent
                .calculator
                .engine
                .tqgnp(constituent.phase_index())?,
            index: constituent.index(),
            status: constituent.status()?,
            name: constituent.name()?,
            charge: constituent.charge()?,
            wmass: constituent.wmass()?,
            stoic: constituent.stoic()?,
            incoming_allowed: constituent.incoming_allowed()?,
            ia: constituent.ia()?,
            a: constituent.a()?,
            ac: constituent.ac()?,
            mu: constituent.mu()?,
            h: constituent.h()?,
            s: constituent.s()?,
            g: constituent.g()?,
            cp: constituent.cp()?,
            v: constituent.v()?,
            hm: constituent.hm()?,
            sm: constituent.sm()?,
            gm: constituent.gm()?,
            cpm: constituent.cpm()?,
            vm: constituent.vm()?,
        })
    }
}
