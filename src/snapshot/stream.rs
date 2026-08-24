use crate::entities::stream::Stream;
use crate::error::ChemAppError;
use crate::snapshot::UnitsSnapshot;

#[derive(Debug, Clone, PartialEq)]
/// Engine-independent copy of one named stream and its active units.
pub struct StreamSnapshot {
    /// Units active when the stream was captured.
    pub units: UnitsSnapshot,
    /// Stream name.
    pub name: String,
    /// Stream temperature.
    pub temperature: f64,
    /// Stream pressure.
    pub pressure: f64,
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

impl StreamSnapshot {
    /// Captures values and current unit labels from a live stream.
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
