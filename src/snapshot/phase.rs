use crate::entities::phase::Phase;
use crate::error::ChemAppError;
use crate::snapshot::{BondSnapshot, ConstituentSnapshot, SpeciesSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseComponentSnapshot {
    pub component_index: usize,
    pub xp: f64,
    pub ap: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseSnapshot {
    pub index: usize,
    pub status: String,
    pub name: String,
    pub model: String,
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
    pub components: Vec<PhaseComponentSnapshot>,
    pub constituents: Vec<ConstituentSnapshot>,
    pub species: Vec<SpeciesSnapshot>,
    pub bonds: Vec<BondSnapshot>,
}

impl PhaseSnapshot {
    pub fn new(phase: &Phase<'_>) -> Result<Self, ChemAppError> {
        let ac = phase.ac()?;
        Self::new_with_activity(phase, ac)
    }

    /// Builds the deep phase snapshot using the activity already queried by
    /// the root stable-phase filter.
    pub(crate) fn new_with_activity(phase: &Phase<'_>, ac: f64) -> Result<Self, ChemAppError> {
        let mut components = Vec::new();
        for component in phase.calculator.components()? {
            components.push(PhaseComponentSnapshot {
                component_index: component.index(),
                xp: component.xp(phase)?,
                ap: component.ap(phase)?,
            });
        }

        let constituents = phase
            .constituents()?
            .map(|constituent| constituent.snapshot())
            .collect::<Result<Vec<_>, _>>()?;
        let species = phase
            .species()?
            .map(|species| species.snapshot())
            .collect::<Result<Vec<_>, _>>()?;
        let bonds = phase
            .bonds()?
            .map(|bond| bond.snapshot())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            index: phase.index(),
            status: phase.status()?,
            name: phase.name()?,
            model: phase.model()?,
            a: phase.a()?,
            ac,
            mu: phase.mu()?,
            h: phase.h()?,
            s: phase.s()?,
            g: phase.g()?,
            cp: phase.cp()?,
            v: phase.v()?,
            hm: phase.hm()?,
            sm: phase.sm()?,
            gm: phase.gm()?,
            cpm: phase.cpm()?,
            vm: phase.vm()?,
            components,
            constituents,
            species,
            bonds,
        })
    }
}
