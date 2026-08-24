use crate::entities::component::SystemComponent;
use crate::error::ChemAppError;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemComponentSnapshot {
    pub index: usize,
    pub name: String,
    pub wmass: f64,
    pub stoic: Vec<f64>,
    pub ia: f64,
    pub a: f64,
    pub ac: f64,
    pub mu: f64,
    pub x: f64,
}

impl SystemComponentSnapshot {
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
