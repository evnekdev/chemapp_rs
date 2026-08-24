use crate::entities::phase::Phase;
use crate::error::ChemAppError;
use crate::snapshot::{BondSnapshot, ConstituentSnapshot, SpeciesSnapshot};

#[derive(Debug, Clone, PartialEq)]
/// One system component's contribution inside a phase snapshot.
pub struct PhaseComponentSnapshot {
    /// One-based system-component index.
    pub component_index: usize,
    /// Component mole fraction in the phase (`XP`).
    pub xp: f64,
    /// Component amount in the phase (`AP`).
    pub ap: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// Deep, engine-independent copy of one phase and its descendants.
pub struct PhaseSnapshot {
    /// One-based phase index.
    pub index: usize,
    /// Native phase status text.
    pub status: String,
    /// Phase name.
    pub name: String,
    /// ChemApp model identifier.
    pub model: String,
    /// Phase amount (`A`).
    pub a: f64,
    /// Phase activity (`AC`).
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
    /// Per-component phase values in native component order.
    pub components: Vec<PhaseComponentSnapshot>,
    /// Ordinary phase-constituent snapshots.
    pub constituents: Vec<ConstituentSnapshot>,
    /// Model-aware sublattice-species snapshots.
    pub species: Vec<SpeciesSnapshot>,
    /// Model-aware pair or quadruplet snapshots from TQBOND.
    pub bonds: Vec<BondSnapshot>,
}

impl PhaseSnapshot {
    /// Captures a live phase and all applicable descendants.
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
