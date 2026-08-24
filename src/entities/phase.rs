//! Fallible live access to a ChemApp phase.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::iterator::{BondIterator, ConstituentIterator, SpeciesIterator};
use crate::snapshot::PhaseSnapshot;
use crate::{Interaction, InteractionChannel, InteractionDescriptorCrossCheck};

/// One-based ChemApp phase identity tied to a live calculator.
#[derive(Debug)]
pub struct Phase<'a> {
    pub(crate) calculator: &'a Calculator,
    pub(crate) index: usize,
}

impl<'a> Phase<'a> {
    /// Creates a live view for a one-based phase index.
    pub fn new(calculator: &'a Calculator, index: usize) -> Self {
        Self { calculator, index }
    }

    /// Returns the one-based ChemApp phase index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Copies current phase, component, constituent, species, and bond values.
    pub fn snapshot(&self) -> Result<PhaseSnapshot, ChemAppError> {
        PhaseSnapshot::new(self)
    }

    /// Formats this phase using the shared live/snapshot table schema.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_phase_table(self)
    }

    /// Reports whether the phase is stable in the current calculated state.
    pub fn is_stable(&self) -> Result<bool, ChemAppError> {
        Ok(crate::snapshot::is_stable_phase_activity(self.ac()?))
    }

    /// Enumerates sublattice constituents for a solution phase.
    ///
    /// `TQMODL == PURE` is ChemApp's documented non-mixture marker and yields
    /// no species. Other model codes are queried structurally through
    /// `TQNOSL`/`TQNOLC`; model-name prefixes are not an applicability rule.
    pub fn species(&self) -> Result<SpeciesIterator<'a>, ChemAppError> {
        SpeciesIterator::new(self.calculator, self.index)
    }

    /// Enumerates TQBOND entities only for models to which TQBOND applies.
    pub fn bonds(&self) -> Result<BondIterator<'a>, ChemAppError> {
        BondIterator::new(self.calculator, self.index)
    }

    /// Enumerates ordinary phase constituents.
    pub fn constituents(&self) -> Result<ConstituentIterator<'a>, ChemAppError> {
        ConstituentIterator::new(self.calculator, self.index)
    }

    /// Inspect one channel of static interaction-model data for this phase.
    ///
    /// Every row retains its raw TQLPAR descriptor and TQGPAR values. Parsing
    /// and model-aware name resolution are additive and never silently drop an
    /// unknown descriptor.
    pub fn interactions(
        &self,
        channel: InteractionChannel,
    ) -> Result<Vec<Interaction>, ChemAppError> {
        crate::interactions::load_phase_interactions(&self.calculator.engine, self.index, channel)
    }

    /// Inspect one channel with independent ASCII-DAT structural evidence.
    ///
    /// Native TQLPAR parsing and live TQGPAR values are always retained. A DAT
    /// difference replaces effective structure only for an explicit validated
    /// native defect; errors and ordinary disagreements remain diagnostics.
    pub fn interactions_with_cross_check(
        &self,
        channel: InteractionChannel,
        cross_check: &dyn InteractionDescriptorCrossCheck,
    ) -> Result<Vec<Interaction>, ChemAppError> {
        crate::interactions::load_phase_interactions_with_cross_check(
            &self.calculator.engine,
            self.index,
            channel,
            Some(cross_check),
        )
    }

    /// Compatibility forwarding name for [`Self::interactions_with_cross_check`].
    #[deprecated(note = "use interactions_with_cross_check")]
    pub fn interactions_with_recovery(
        &self,
        channel: InteractionChannel,
        cross_check: &dyn InteractionDescriptorCrossCheck,
    ) -> Result<Vec<Interaction>, ChemAppError> {
        self.interactions_with_cross_check(channel, cross_check)
    }

    /// Inspect excess Gibbs-energy interactions for this phase.
    pub fn gibbs_interactions(&self) -> Result<Vec<Interaction>, ChemAppError> {
        self.interactions(InteractionChannel::GibbsExcess)
    }

    /// Inspect excess magnetic interactions for this phase.
    pub fn magnetic_interactions(&self) -> Result<Vec<Interaction>, ChemAppError> {
        self.interactions(InteractionChannel::Magnetic)
    }

    /// Reports whether the phase index exists in the loaded system.
    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        Ok(self.index > 0 && self.index <= self.calculator.engine.tqnop()?)
    }

    /// Reports whether ChemApp identifies this as a stoichiometric (`PURE`) phase.
    pub fn is_stoic(&self) -> Result<bool, ChemAppError> {
        Ok(self.model()? == "PURE")
    }

    /// Returns the current phase status text.
    pub fn status(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgsp(self.index)
    }

    /// Returns the phase name.
    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnp(self.index)
    }

    /// Returns the ChemApp phase-model identifier.
    pub fn model(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqmodl(self.index)
    }

    fn result(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr(option, self.index, 0)
    }

    /// Returns phase amount (`A`).
    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.result("A")
    }
    /// Returns phase activity (`AC`).
    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.result("AC")
    }
    /// Returns phase chemical potential (`MU`).
    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.result("MU")
    }
    /// Returns phase enthalpy (`H`).
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.result("H")
    }
    /// Returns phase entropy (`S`).
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.result("S")
    }
    /// Returns phase Gibbs energy (`G`).
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.result("G")
    }
    /// Returns phase heat capacity (`CP`).
    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.result("CP")
    }
    /// Returns phase volume (`V`).
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.result("V")
    }
    /// Returns molar phase enthalpy (`HM`).
    pub fn hm(&self) -> Result<f64, ChemAppError> {
        self.result("HM")
    }
    /// Returns molar phase entropy (`SM`).
    pub fn sm(&self) -> Result<f64, ChemAppError> {
        self.result("SM")
    }
    /// Returns molar phase Gibbs energy (`GM`).
    pub fn gm(&self) -> Result<f64, ChemAppError> {
        self.result("GM")
    }
    /// Returns molar phase heat capacity (`CPM`).
    pub fn cpm(&self) -> Result<f64, ChemAppError> {
        self.result("CPM")
    }
    /// Returns molar phase volume (`VM`).
    pub fn vm(&self) -> Result<f64, ChemAppError> {
        self.result("VM")
    }
}
