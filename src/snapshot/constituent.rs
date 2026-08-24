use crate::entities::constituent::Constituent;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
/// Engine-independent copy of one phase constituent at a calculated state.
pub struct ConstituentSnapshot {
    /// One-based parent phase index.
    pub phase_index: usize,
    /// Parent phase name.
    pub phase_name: String,
    /// One-based constituent index within the phase.
    pub index: usize,
    /// Native status text.
    pub status: String,
    /// Constituent name.
    pub name: String,
    /// Real-valued constituent charge.
    pub charge: f64,
    /// Molar mass.
    pub wmass: f64,
    /// Stoichiometry in system-component order.
    pub stoic: Vec<f64>,
    /// Whether incoming material is allowed.
    pub incoming_allowed: bool,
    /// Entered amount (`IA`).
    pub ia: f64,
    /// Calculated amount (`A`).
    pub a: f64,
    /// Activity (`AC`).
    pub ac: f64,
    /// Chemical potential (`MU`).
    pub mu: f64,
    /// Enthalpy (`H`).
    pub h: f64,
    /// Entropy (`S`).
    pub s: f64,
    /// Gibbs energy (`G`).
    pub g: f64,
    /// Heat capacity (`CP`).
    pub cp: f64,
    /// Volume (`V`).
    pub v: f64,
    /// Molar enthalpy (`HM`).
    pub hm: f64,
    /// Molar entropy (`SM`).
    pub sm: f64,
    /// Molar Gibbs energy (`GM`).
    pub gm: f64,
    /// Molar heat capacity (`CPM`).
    pub cpm: f64,
    /// Molar volume (`VM`).
    pub vm: f64,
}

impl ConstituentSnapshot {
    /// Captures all exposed values from a live constituent.
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
