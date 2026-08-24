//! Engine-independent snapshots of one calculated ChemApp state.

pub mod bond;
pub mod component;
pub mod constituent;
pub mod phase;
pub mod species;
pub mod stream;
pub mod system;

use std::fmt;

use crate::calculator::Calculator;
use crate::error::ChemAppError;

pub use bond::{BondSnapshot, BondSnapshotKind, PairMemberSnapshot, QuadrupletMemberSnapshot};
pub use component::SystemComponentSnapshot;
pub use constituent::ConstituentSnapshot;
pub use phase::{PhaseComponentSnapshot, PhaseSnapshot};
pub use species::SpeciesSnapshot;
pub use stream::StreamSnapshot;
pub use system::SystemSnapshot;

/// The one authoritative phase-stability criterion for snapshots and tables.
pub const STABLE_PHASE_ACTIVITY_THRESHOLD: f64 = 0.9999;

pub fn is_stable_phase_activity(activity: f64) -> bool {
    activity > STABLE_PHASE_ACTIVITY_THRESHOLD
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotOptions {
    pub stable_only: bool,
}

impl SnapshotOptions {
    pub const fn all() -> Self {
        Self { stable_only: false }
    }
    pub const fn stable_only() -> Self {
        Self { stable_only: true }
    }
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self::all()
    }
}

/// Active ChemApp units captured once for a whole calculator snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitsSnapshot {
    pub temperature: String,
    pub pressure: String,
    pub volume: String,
    pub energy: String,
    pub amount: String,
}

impl UnitsSnapshot {
    pub(crate) fn new(calculator: &Calculator) -> Result<Self, ChemAppError> {
        Ok(Self {
            temperature: calculator.engine.tqgsu("Temperature")?,
            pressure: calculator.engine.tqgsu("Pressure")?,
            volume: calculator.engine.tqgsu("Volume")?,
            energy: calculator.engine.tqgsu("Energy")?,
            amount: calculator.engine.tqgsu("Amount")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalculatorSnapshot {
    options: SnapshotOptions,
    units: UnitsSnapshot,
    system: SystemSnapshot,
    components: Vec<SystemComponentSnapshot>,
    phases: Vec<PhaseSnapshot>,
}

impl CalculatorSnapshot {
    pub fn new(calculator: &Calculator) -> Result<Self, ChemAppError> {
        Self::new_with_options(calculator, SnapshotOptions::default())
    }

    pub fn new_with_options(
        calculator: &Calculator,
        options: SnapshotOptions,
    ) -> Result<Self, ChemAppError> {
        let units = UnitsSnapshot::new(calculator)?;
        let system = calculator.system().snapshot()?;
        let components = calculator
            .components()?
            .map(|component| component.snapshot())
            .collect::<Result<Vec<_>, _>>()?;

        let mut phases = Vec::new();
        for phase in calculator.phases()? {
            // Stability is activity-based and is tested before any deep phase
            // descendants are queried.
            let ac = phase.ac()?;
            if options.stable_only && !is_stable_phase_activity(ac) {
                continue;
            }
            phases.push(PhaseSnapshot::new_with_activity(&phase, ac)?);
        }

        Ok(Self {
            options,
            units,
            system,
            components,
            phases,
        })
    }

    pub fn units(&self) -> &UnitsSnapshot {
        &self.units
    }
    pub fn options(&self) -> SnapshotOptions {
        self.options
    }
    pub fn system(&self) -> &SystemSnapshot {
        &self.system
    }
    pub fn components(&self) -> &[SystemComponentSnapshot] {
        &self.components
    }
    pub fn phases(&self) -> &[PhaseSnapshot] {
        &self.phases
    }
}

impl fmt::Display for CalculatorSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&crate::table::snapshot_report(self))
    }
}

impl fmt::Display for StreamSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&crate::table::stream_snapshot_table(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stability_boundary_is_strict_and_activity_based() {
        assert!(!is_stable_phase_activity(0.0));
        assert!(!is_stable_phase_activity(
            STABLE_PHASE_ACTIVITY_THRESHOLD - 1.0e-12
        ));
        assert!(!is_stable_phase_activity(STABLE_PHASE_ACTIVITY_THRESHOLD));
        assert!(is_stable_phase_activity(
            STABLE_PHASE_ACTIVITY_THRESHOLD + 1.0e-12
        ));
        assert!(is_stable_phase_activity(1.0));
    }

    #[test]
    fn snapshot_options_are_explicit() {
        assert_eq!(SnapshotOptions::default(), SnapshotOptions::all());
        assert!(!SnapshotOptions::all().stable_only);
        assert!(SnapshotOptions::stable_only().stable_only);
    }
}
