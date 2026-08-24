use crate::entities::stream::Stream;
use crate::error::ChemAppError;
use crate::snapshot::UnitsSnapshot;

#[derive(Debug, Clone, PartialEq)]
pub struct StreamSnapshot {
    pub units: UnitsSnapshot,
    pub name: String,
    pub temperature: f64,
    pub pressure: f64,
    pub cp: f64,
    pub h: f64,
    pub s: f64,
    pub g: f64,
    pub v: f64,
}

impl StreamSnapshot {
    pub fn new(stream: &Stream<'_>) -> Result<Self, ChemAppError> {
        Ok(Self {
            units: UnitsSnapshot::new(stream.calculator)?,
            name: stream.name().to_owned(),
            temperature: stream.temperature(),
            pressure: stream.pressure(),
            cp: stream.cp()?,
            h: stream.h()?,
            s: stream.s()?,
            g: stream.g()?,
            v: stream.v()?,
        })
    }
}
