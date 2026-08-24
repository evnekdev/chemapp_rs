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
    pub fn new(calculator: &'a Calculator, index: usize) -> Self {
        Self { calculator, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn snapshot(&self) -> Result<PhaseSnapshot, ChemAppError> {
        PhaseSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_phase_table(self)
    }

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

    pub fn is_valid(&self) -> Result<bool, ChemAppError> {
        Ok(self.index > 0 && self.index <= self.calculator.engine.tqnop()?)
    }

    pub fn is_stoic(&self) -> Result<bool, ChemAppError> {
        Ok(self.model()? == "PURE")
    }

    pub fn status(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgsp(self.index)
    }

    pub fn name(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqgnp(self.index)
    }

    pub fn model(&self) -> Result<String, ChemAppError> {
        self.calculator.engine.tqmodl(self.index)
    }

    fn result(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqgetr(option, self.index, 0)
    }

    pub fn a(&self) -> Result<f64, ChemAppError> {
        self.result("A")
    }
    pub fn ac(&self) -> Result<f64, ChemAppError> {
        self.result("AC")
    }
    pub fn mu(&self) -> Result<f64, ChemAppError> {
        self.result("MU")
    }
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.result("H")
    }
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.result("S")
    }
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.result("G")
    }
    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.result("CP")
    }
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.result("V")
    }
    pub fn hm(&self) -> Result<f64, ChemAppError> {
        self.result("HM")
    }
    pub fn sm(&self) -> Result<f64, ChemAppError> {
        self.result("SM")
    }
    pub fn gm(&self) -> Result<f64, ChemAppError> {
        self.result("GM")
    }
    pub fn cpm(&self) -> Result<f64, ChemAppError> {
        self.result("CPM")
    }
    pub fn vm(&self) -> Result<f64, ChemAppError> {
        self.result("VM")
    }
}
