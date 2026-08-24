use crate::entities::component::SystemComponent;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
/// Engine-independent copy of one system component at a calculated state.
pub struct SystemComponentSnapshot {
    /// One-based system-component index.
    pub index: usize,
    /// Component name.
    pub name: String,
    /// Molar mass.
    pub wmass: f64,
    /// Stoichiometry in system-component order.
    pub stoic: Vec<f64>,
    /// Entered amount (`IA`).
    pub ia: f64,
    /// Calculated amount (`A`).
    pub a: f64,
    /// Activity (`AC`).
    pub ac: f64,
    /// Chemical potential (`MU`).
    pub mu: f64,
    /// System mole fraction (`X`).
    pub x: f64,
}

impl SystemComponentSnapshot {
    /// Captures all exposed values from a live component.
    pub fn new(component: &SystemComponent<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            index: component.index(),
            name: component.name()?,
            wmass: component.wmass()?,
            stoic: component.stoic()?,
            ia: component.ia()?,
            a: component.a()?,
            ac: component.ac()?,
            mu: component.mu()?,
            x: component.x()?,
        })
    }
}
