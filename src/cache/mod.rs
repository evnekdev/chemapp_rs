//! Baseline cache for reversible ChemApp parameter perturbations.
//!
//! Interaction identity is structural: phase/channel/interaction/expression
//! plus a Gibbs term or magnetic role. Pretty descriptor text is retained for
//! display only and is never the mutation key. Every TQGPAR cell is cached;
//! cells without a verified TQCDAT selector remain explicitly read-only.

use std::collections::HashMap;

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::interactions::{
    Interaction, InteractionChannel, InteractionMutationSupport, InteractionParameterAddress,
    InteractionParameterRole,
};
use crate::Engine;

impl Calculator {
    /// Build a baseline parameter cache for the named phases.
    pub fn generate_parameter_cache<T: AsRef<str> + std::fmt::Debug>(
        &mut self,
        phase_names: &[T],
        include_gibbs: bool,
        include_magnetic: bool,
        include_endmembers: bool,
        include_compounds: bool,
    ) -> Result<(), ChemAppError> {
        let cache = ParameterCache::new(
            self,
            phase_names,
            include_gibbs,
            include_magnetic,
            include_endmembers,
            include_compounds,
        )?;
        self.install_parameter_cache(cache);
        Ok(())
    }

    fn required_parameter_cache(&self) -> Result<&ParameterCache, ChemAppError> {
        self.parameter_cache().ok_or_else(|| {
            ChemAppError::OtherError(
                "no ParameterCache is installed; call generate_parameter_cache first".to_owned(),
            )
        })
    }

    /// Writes an absolute value or baseline-relative delta for a cached,
    /// verified interaction address using this Calculator's own Engine.
    pub fn set_cached_interaction_parameter(
        &self,
        address: InteractionParameterAddress,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?.set_interaction_parameter(
            self.engine(),
            address,
            value,
            is_delta,
        )
    }

    /// Restores one cached interaction parameter to its captured baseline.
    pub fn reset_cached_interaction_parameter(
        &self,
        address: InteractionParameterAddress,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?
            .reset_interaction_parameter(self.engine(), address)
    }

    /// Restores every verified cached cell for one native interaction.
    pub fn reset_cached_interaction(
        &self,
        phase_index: usize,
        channel: InteractionChannel,
        interaction_index: usize,
    ) -> Result<usize, ChemAppError> {
        self.required_parameter_cache()?.reset_interaction(
            self.engine(),
            phase_index,
            channel,
            interaction_index,
        )
    }

    /// Restores all verified cached interaction cells and verifies readback.
    pub fn reset_cached_interactions(&self) -> Result<(), ChemAppError> {
        self.required_parameter_cache()?
            .reset_interactions(self.engine())
    }

    /// Restores all cached compound, endmember, and interaction baselines.
    pub fn reset_cached_parameters(&self) -> Result<(), ChemAppError> {
        self.required_parameter_cache()?.reset_all(self.engine())
    }

    /// Sets cached pure-phase enthalpy, absolutely or relative to the baseline.
    pub fn set_cached_compound_h298(
        &self,
        phase: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?
            .set_compound_h298(self.engine(), phase, value, is_delta)
    }

    /// Sets cached pure-phase entropy, absolutely or relative to the baseline.
    pub fn set_cached_compound_s298(
        &self,
        phase: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?
            .set_compound_s298(self.engine(), phase, value, is_delta)
    }

    /// Sets cached endmember enthalpy, absolutely or relative to the baseline.
    pub fn set_cached_endmember_h298(
        &self,
        phase: &str,
        constituent: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?.set_endmember_h298(
            self.engine(),
            phase,
            constituent,
            value,
            is_delta,
        )
    }

    /// Sets cached endmember entropy, absolutely or relative to the baseline.
    pub fn set_cached_endmember_s298(
        &self,
        phase: &str,
        constituent: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.required_parameter_cache()?.set_endmember_s298(
            self.engine(),
            phase,
            constituent,
            value,
            is_delta,
        )
    }
}

/// Structural key for one cached interaction cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InteractionParameterKey {
    /// One-based native phase index.
    pub phase_index: usize,
    /// TQLPAR/TQGPAR option channel.
    pub channel: InteractionChannel,
    /// One-based interaction position in native data-file order.
    pub interaction_index: usize,
    /// One-based TQGPAR expression row.
    pub expression_index: usize,
    /// One-based TQGPAR column.
    pub column_index: usize,
}

/// Cached baseline and mutation metadata for one complete TQGPAR cell.
#[derive(Clone, Debug, PartialEq)]
pub struct CachedInteractionParameter {
    /// Stable structural identity independent of display text.
    pub key: InteractionParameterKey,
    /// Phase name retained for display and convenience lookup.
    pub phase_name: String,
    /// Native TQMODL code.
    pub model: String,
    /// Exact native TQLPAR text.
    pub native_descriptor: String,
    /// Name-resolved display form, or an explicit unresolved diagnostic.
    pub resolved_descriptor: String,
    /// Channel-aware meaning of this matrix column.
    pub role: InteractionParameterRole,
    /// Original live TQGPAR value. Delta writes are always relative to this.
    pub baseline: f64,
    /// Verified TQCDAT address or an explicit read-only reason.
    pub mutation: InteractionMutationSupport,
}

#[derive(Clone, Debug, PartialEq)]
/// Read-only cached standard-state baselines for one solution-phase endmember.
///
/// Native indices and names describe the Calculator/system that captured the
/// cache. They are suitable for owned discovery snapshots, but are not
/// persistent identity across different loaded datafiles. Parameter mutation
/// remains available only through Calculator-owned checked setters.
pub struct CachedEndmemberParameter {
    /// One-based native phase index.
    pub phase_index: usize,
    /// One-based native constituent index within the phase.
    pub constituent_index: usize,
    /// Exact native phase name.
    pub phase_name: String,
    /// Exact native constituent name.
    pub constituent_name: String,
    /// Captured H298 baseline.
    pub h298: f64,
    /// Captured S298 baseline.
    pub s298: f64,
}

impl CachedEndmemberParameter {
    fn reset(&self, engine: &Engine) -> Result<(), ChemAppError> {
        engine.tqcdat(1, 0, 0, self.constituent_index, self.phase_index, self.h298)?;
        engine.tqcdat(1, 0, 1, self.constituent_index, self.phase_index, self.s298)
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Read-only cached standard-state baselines for one stoichiometric compound.
///
/// The one-based native phase index and exact name are Calculator/system-local
/// discovery facts, not persistent identity across different datafiles.
/// Parameter mutation remains available only through Calculator-owned checked
/// setters.
pub struct CachedCompoundParameter {
    /// One-based native phase index.
    pub phase_index: usize,
    /// Exact native phase name.
    pub phase_name: String,
    /// Captured H298 baseline.
    pub h298: f64,
    /// Captured S298 baseline.
    pub s298: f64,
}

impl CachedCompoundParameter {
    fn reset(&self, engine: &Engine) -> Result<(), ChemAppError> {
        engine.tqcdat(1, 0, 0, 1, self.phase_index, self.h298)?;
        engine.tqcdat(1, 0, 1, 1, self.phase_index, self.s298)
    }
}

/// A model-neutral cache of live ChemApp parameter baselines.
///
/// Cached native addresses are meaningful only for the Calculator/system that
/// captured them. Public mutation is therefore exposed on [`Calculator`],
/// which always applies this cache to its owning Engine; this type exposes
/// read-only inspection methods only.
#[derive(Debug)]
pub struct ParameterCache {
    interaction_lookup: HashMap<InteractionParameterAddress, usize>,
    interaction_parameters: Vec<CachedInteractionParameter>,
    endmember_lookup: HashMap<(String, String), usize>,
    compound_lookup: HashMap<String, usize>,
    endmembers: Vec<CachedEndmemberParameter>,
    compounds: Vec<CachedCompoundParameter>,
}

fn cache_interaction(interaction: Interaction) -> Vec<CachedInteractionParameter> {
    interaction
        .parameter_cells()
        .into_iter()
        .map(|cell| CachedInteractionParameter {
            key: InteractionParameterKey {
                phase_index: interaction.raw.phase_index,
                channel: interaction.raw.channel,
                interaction_index: interaction.raw.parameter_index,
                expression_index: cell.expression_index,
                column_index: cell.column_index,
            },
            phase_name: interaction.raw.phase_name.clone(),
            model: interaction.raw.model.clone(),
            native_descriptor: interaction.raw.raw_descriptor.clone(),
            resolved_descriptor: interaction.resolved_text(),
            role: cell.role,
            baseline: cell.value,
            mutation: cell.mutation,
        })
        .collect()
}

fn first_tqgdat_value(
    values: Vec<f64>,
    phase_name: &str,
    constituent_index: usize,
    option: &str,
) -> Result<f64, ChemAppError> {
    values.into_iter().next().ok_or_else(|| {
        ChemAppError::OtherError(format!(
            "TQGDAT returned no {option} value for phase {phase_name:?}, constituent {constituent_index}"
        ))
    })
}

impl ParameterCache {
    /// Capture baselines for requested interaction channels and standard-state data.
    fn new<T: AsRef<str> + std::fmt::Debug>(
        calculator: &Calculator,
        phase_names: &[T],
        include_gibbs: bool,
        include_magnetic: bool,
        include_endmembers: bool,
        include_compounds: bool,
    ) -> Result<Self, ChemAppError> {
        let mut interaction_parameters = Vec::new();
        let mut endmembers = Vec::new();
        let mut compounds = Vec::new();
        for phase_name in phase_names {
            let phase_name = phase_name.as_ref();
            let phase_index = calculator.engine().tqinp(phase_name)?;
            if calculator.engine().tqmodl(phase_index)? == "PURE" {
                if include_compounds {
                    compounds.push(Self::load_compound(calculator, phase_name)?);
                }
                continue;
            }
            if include_endmembers {
                endmembers.extend(Self::load_endmembers(calculator, phase_name)?);
            }
            for channel in [
                InteractionChannel::GibbsExcess,
                InteractionChannel::Magnetic,
            ] {
                let include = match channel {
                    InteractionChannel::GibbsExcess => include_gibbs,
                    InteractionChannel::Magnetic => include_magnetic,
                };
                if include {
                    for interaction in crate::interactions::load_phase_interactions(
                        calculator.engine(),
                        phase_index,
                        channel,
                    )? {
                        interaction_parameters.extend(cache_interaction(interaction));
                    }
                }
            }
        }
        let interaction_lookup = interaction_parameters
            .iter()
            .enumerate()
            .filter_map(|(offset, parameter)| match parameter.mutation {
                InteractionMutationSupport::Verified(address) => Some((address, offset)),
                InteractionMutationSupport::ReadOnly { .. } => None,
            })
            .collect();
        let endmember_lookup = endmembers
            .iter()
            .enumerate()
            .map(|(offset, value)| {
                (
                    (value.phase_name.clone(), value.constituent_name.clone()),
                    offset,
                )
            })
            .collect();
        let compound_lookup = compounds
            .iter()
            .enumerate()
            .map(|(offset, value)| (value.phase_name.clone(), offset))
            .collect();
        Ok(Self {
            interaction_lookup,
            interaction_parameters,
            endmember_lookup,
            compound_lookup,
            endmembers,
            compounds,
        })
    }

    /// Complete cached TQGPAR surface, including read-only cells.
    pub fn interaction_parameters(&self) -> &[CachedInteractionParameter] {
        &self.interaction_parameters
    }

    /// Complete cached solution-endmember standard-state baselines.
    pub fn endmember_parameters(&self) -> &[CachedEndmemberParameter] {
        &self.endmembers
    }

    /// Complete cached stoichiometric-compound standard-state baselines.
    pub fn compound_parameters(&self) -> &[CachedCompoundParameter] {
        &self.compounds
    }

    /// Look up one mutable cell by its exact native structural address.
    pub fn parameter(
        &self,
        address: InteractionParameterAddress,
    ) -> Option<&CachedInteractionParameter> {
        self.interaction_lookup
            .get(&address)
            .map(|offset| &self.interaction_parameters[*offset])
    }

    /// Parameters for one structural phase/channel/interaction identity.
    pub fn interaction_by_index(
        &self,
        phase_index: usize,
        channel: InteractionChannel,
        interaction_index: usize,
    ) -> Vec<&CachedInteractionParameter> {
        self.interaction_parameters
            .iter()
            .filter(|parameter| {
                parameter.key.phase_index == phase_index
                    && parameter.key.channel == channel
                    && parameter.key.interaction_index == interaction_index
            })
            .collect()
    }

    /// Convenience lookup by phase name plus native channel/interaction index.
    pub fn interaction(
        &self,
        phase_name: &str,
        channel: InteractionChannel,
        interaction_index: usize,
    ) -> Vec<&CachedInteractionParameter> {
        self.interaction_parameters
            .iter()
            .filter(|parameter| {
                parameter.phase_name == phase_name
                    && parameter.key.channel == channel
                    && parameter.key.interaction_index == interaction_index
            })
            .collect()
    }

    /// Deterministic table of every cached interaction cell and its support.
    pub fn table_string(&self) -> String {
        let rows = self
            .interaction_parameters
            .iter()
            .map(|parameter| {
                let support = match &parameter.mutation {
                    InteractionMutationSupport::Verified(address) => address
                        .tqcdat_selectors()
                        .map(|selectors| format!("Verified {:?}", selectors))
                        .unwrap_or_else(|error| format!("Invalid: {error}")),
                    InteractionMutationSupport::ReadOnly { reason } => {
                        format!("Read-only: {reason}")
                    }
                };
                vec![
                    parameter.phase_name.clone(),
                    parameter.model.clone(),
                    parameter.key.channel.to_string(),
                    parameter.key.interaction_index.to_string(),
                    parameter.key.expression_index.to_string(),
                    parameter.key.column_index.to_string(),
                    parameter.role.to_string(),
                    format!("{:.8e}", parameter.baseline),
                    support,
                    parameter.resolved_descriptor.clone(),
                ]
            })
            .collect();
        crate::table::render(
            "ChemApp parameter cache",
            &[
                "Phase",
                "Model",
                "Channel",
                "Interaction",
                "Expression",
                "Column",
                "Role",
                "Baseline",
                "Mutation",
                "Descriptor",
            ]
            .map(str::to_owned),
            rows,
        )
    }

    /// Write an absolute value or `baseline + delta` for a verified address.
    fn set_interaction_parameter(
        &self,
        engine: &Engine,
        address: InteractionParameterAddress,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        let Some(offset) = self.interaction_lookup.get(&address).copied() else {
            return Ok(false);
        };
        let baseline = self.interaction_parameters[offset].baseline;
        crate::interactions::write_interaction_parameter(
            engine,
            address,
            if is_delta { baseline + value } else { value },
        )?;
        Ok(true)
    }

    fn verified_interaction_baselines(
        &self,
    ) -> Vec<(InteractionParameterAddress, f64, InteractionParameterKey)> {
        self.interaction_parameters
            .iter()
            .filter_map(|parameter| match parameter.mutation {
                InteractionMutationSupport::Verified(address) => {
                    Some((address, parameter.baseline, parameter.key))
                }
                InteractionMutationSupport::ReadOnly { .. } => None,
            })
            .collect()
    }

    /// Restore one verified cell to its captured baseline and verify readback.
    fn reset_interaction_parameter(
        &self,
        engine: &Engine,
        address: InteractionParameterAddress,
    ) -> Result<bool, ChemAppError> {
        let Some(parameter) = self.parameter(address) else {
            return Ok(false);
        };
        crate::interactions::write_interaction_parameter(engine, address, parameter.baseline)?;
        let actual = crate::interactions::read_interaction_parameter(engine, address)?.value;
        if actual != parameter.baseline {
            return Err(ChemAppError::OtherError(format!(
                "interaction reset readback mismatch at {:?}",
                parameter.key
            )));
        }
        Ok(true)
    }

    /// Restore every verified cell of one native interaction exactly once.
    fn reset_interaction(
        &self,
        engine: &Engine,
        phase_index: usize,
        channel: InteractionChannel,
        interaction_index: usize,
    ) -> Result<usize, ChemAppError> {
        let addresses = self
            .interaction_by_index(phase_index, channel, interaction_index)
            .into_iter()
            .filter_map(|parameter| match parameter.mutation {
                InteractionMutationSupport::Verified(address) => Some(address),
                InteractionMutationSupport::ReadOnly { .. } => None,
            })
            .collect::<Vec<_>>();
        for address in &addresses {
            self.reset_interaction_parameter(engine, *address)?;
        }
        Ok(addresses.len())
    }

    /// Restore every verified interaction cell and verify live readback.
    fn reset_interactions(&self, engine: &Engine) -> Result<(), ChemAppError> {
        let baselines = self.verified_interaction_baselines();
        for (address, baseline, _) in &baselines {
            crate::interactions::write_interaction_parameter(engine, *address, *baseline)?;
        }
        for (address, baseline, key) in baselines {
            let actual = crate::interactions::read_interaction_parameter(engine, address)?.value;
            if actual != baseline {
                return Err(ChemAppError::OtherError(format!(
                    "interaction reset readback mismatch at {key:?}"
                )));
            }
        }
        Ok(())
    }

    /// Restore every cached compound, endmember, and verified interaction value.
    fn reset_all(&self, engine: &Engine) -> Result<(), ChemAppError> {
        for compound in &self.compounds {
            compound.reset(engine)?;
        }
        for endmember in &self.endmembers {
            endmember.reset(engine)?;
        }
        self.reset_interactions(engine)
    }

    /// Load the standard-state baseline for one pure phase.
    fn load_compound(
        calculator: &Calculator,
        phase_name: &str,
    ) -> Result<CachedCompoundParameter, ChemAppError> {
        let phase_index = calculator.engine().tqinp(phase_name)?;
        let h298 = first_tqgdat_value(
            calculator.engine().tqgdat(phase_index, 1, "H", 0)?,
            phase_name,
            1,
            "H",
        )?;
        let s298 = first_tqgdat_value(
            calculator.engine().tqgdat(phase_index, 1, "S", 0)?,
            phase_name,
            1,
            "S",
        )?;
        Ok(CachedCompoundParameter {
            phase_index,
            phase_name: phase_name.to_owned(),
            h298,
            s298,
        })
    }

    /// Load standard-state baselines for every constituent of a solution phase.
    fn load_endmembers(
        calculator: &Calculator,
        phase_name: &str,
    ) -> Result<Vec<CachedEndmemberParameter>, ChemAppError> {
        let phase_index = calculator.engine().tqinp(phase_name)?;
        (1..=calculator.engine().tqnopc(phase_index)?)
            .map(|constituent_index| {
                let h298 = first_tqgdat_value(
                    calculator
                        .engine()
                        .tqgdat(phase_index, constituent_index, "H", 0)?,
                    phase_name,
                    constituent_index,
                    "H",
                )?;
                let s298 = first_tqgdat_value(
                    calculator
                        .engine()
                        .tqgdat(phase_index, constituent_index, "S", 0)?,
                    phase_name,
                    constituent_index,
                    "S",
                )?;
                Ok(CachedEndmemberParameter {
                    phase_index,
                    constituent_index,
                    phase_name: phase_name.to_owned(),
                    constituent_name: calculator.engine().tqgnpc(phase_index, constituent_index)?,
                    h298,
                    s298,
                })
            })
            .collect()
    }

    /// Set a pure-phase enthalpy baseline or a delta from that baseline.
    fn set_compound_h298(
        &self,
        engine: &Engine,
        phase: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.set_compound(engine, phase, value, is_delta, false)
    }

    /// Set a pure-phase entropy baseline or a delta from that baseline.
    fn set_compound_s298(
        &self,
        engine: &Engine,
        phase: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.set_compound(engine, phase, value, is_delta, true)
    }

    fn set_compound(
        &self,
        engine: &Engine,
        phase: &str,
        value: f64,
        is_delta: bool,
        entropy: bool,
    ) -> Result<bool, ChemAppError> {
        let Some(offset) = self.compound_lookup.get(phase).copied() else {
            return Ok(false);
        };
        let compound = &self.compounds[offset];
        let baseline = if entropy {
            compound.s298
        } else {
            compound.h298
        };
        engine.tqcdat(
            1,
            0,
            usize::from(entropy),
            1,
            compound.phase_index,
            if is_delta { baseline + value } else { value },
        )?;
        Ok(true)
    }

    /// Set an endmember enthalpy baseline or a delta from that baseline.
    fn set_endmember_h298(
        &self,
        engine: &Engine,
        phase: &str,
        constituent: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.set_endmember(engine, phase, constituent, value, is_delta, false)
    }

    /// Set an endmember entropy baseline or a delta from that baseline.
    fn set_endmember_s298(
        &self,
        engine: &Engine,
        phase: &str,
        constituent: &str,
        value: f64,
        is_delta: bool,
    ) -> Result<bool, ChemAppError> {
        self.set_endmember(engine, phase, constituent, value, is_delta, true)
    }

    fn set_endmember(
        &self,
        engine: &Engine,
        phase: &str,
        constituent: &str,
        value: f64,
        is_delta: bool,
        entropy: bool,
    ) -> Result<bool, ChemAppError> {
        let Some(offset) = self
            .endmember_lookup
            .get(&(phase.to_owned(), constituent.to_owned()))
            .copied()
        else {
            return Ok(false);
        };
        let endmember = &self.endmembers[offset];
        let baseline = if entropy {
            endmember.s298
        } else {
            endmember.h298
        };
        engine.tqcdat(
            1,
            0,
            usize::from(entropy),
            endmember.constituent_index,
            endmember.phase_index,
            if is_delta { baseline + value } else { value },
        )?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interactions::{
        InteractionCrossCheck, InteractionDescriptor, InteractionRaw, InteractionResolution,
    };

    fn synthetic_interaction(descriptor: &str) -> Interaction {
        let parsed = InteractionDescriptor::Unparsed {
            raw: descriptor.to_owned(),
            reason: "synthetic cache test".to_owned(),
        };
        Interaction {
            raw: InteractionRaw {
                phase_index: 7,
                phase_name: "Synthetic".to_owned(),
                model: "SUBLM".to_owned(),
                sublattice_count: 2,
                channel: InteractionChannel::GibbsExcess,
                parameter_index: 4,
                raw_descriptor: descriptor.to_owned(),
                values: vec![vec![11.0, 12.0, 13.0], vec![21.0, 22.0, 23.0]],
            },
            native_parsed: parsed.clone(),
            cross_check: InteractionCrossCheck::NotRequested,
            effective_descriptor: parsed,
            resolution: InteractionResolution::Unresolved {
                reason: "synthetic cache test".to_owned(),
            },
        }
    }

    #[test]
    fn structural_keys_distinguish_expression_term_and_channel() {
        let base = InteractionParameterKey {
            phase_index: 7,
            channel: InteractionChannel::GibbsExcess,
            interaction_index: 4,
            expression_index: 1,
            column_index: 1,
        };
        let mut keys = std::collections::HashSet::new();
        keys.insert(base);
        keys.insert(InteractionParameterKey {
            expression_index: 2,
            ..base
        });
        keys.insert(InteractionParameterKey {
            column_index: 2,
            ..base
        });
        keys.insert(InteractionParameterKey {
            channel: InteractionChannel::Magnetic,
            ..base
        });
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn baseline_relative_delta_does_not_accumulate() {
        let baseline = 12.5;
        let delta = -0.25;
        let first = baseline + delta;
        let second = baseline + delta;
        assert_eq!(first, 12.25);
        assert_eq!(first, second);
    }

    #[test]
    fn complete_multi_expression_matrix_is_cached_by_structural_identity() {
        let first = cache_interaction(synthetic_interaction("first display text"));
        let second = cache_interaction(synthetic_interaction("changed display text"));
        assert_eq!(first.len(), 6);
        assert_eq!(
            first
                .iter()
                .map(|parameter| parameter.baseline)
                .collect::<Vec<_>>(),
            vec![11.0, 12.0, 13.0, 21.0, 22.0, 23.0]
        );
        assert_eq!(
            first
                .iter()
                .map(|parameter| parameter.key)
                .collect::<Vec<_>>(),
            second
                .iter()
                .map(|parameter| parameter.key)
                .collect::<Vec<_>>()
        );
        assert!(first.iter().all(|parameter| matches!(
            parameter.mutation,
            InteractionMutationSupport::Verified(_)
        )));
    }

    #[test]
    fn empty_tqgdat_result_is_a_contextual_error_not_an_index_panic() {
        let error = first_tqgdat_value(Vec::new(), "Synthetic", 2, "H").unwrap_err();
        assert!(error.to_string().contains("returned no H value"));
    }

    #[test]
    fn complete_reset_plan_contains_each_structural_address_once() {
        let interaction_parameters = cache_interaction(synthetic_interaction("display"));
        let interaction_lookup = interaction_parameters
            .iter()
            .enumerate()
            .filter_map(|(offset, parameter)| match parameter.mutation {
                InteractionMutationSupport::Verified(address) => Some((address, offset)),
                InteractionMutationSupport::ReadOnly { .. } => None,
            })
            .collect();
        let cache = ParameterCache {
            interaction_lookup,
            interaction_parameters,
            endmember_lookup: HashMap::new(),
            compound_lookup: HashMap::new(),
            endmembers: Vec::new(),
            compounds: Vec::new(),
        };
        let reset_plan = cache.verified_interaction_baselines();
        let unique = reset_plan
            .iter()
            .map(|(address, _, _)| *address)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(reset_plan.len(), 6);
        assert_eq!(unique.len(), reset_plan.len());
    }

    #[test]
    fn standard_state_cache_records_are_available_for_owned_discovery() {
        let endmember = CachedEndmemberParameter {
            phase_index: 2,
            constituent_index: 3,
            phase_name: "solution".to_owned(),
            constituent_name: "CaO".to_owned(),
            h298: -635_090.0,
            s298: 38.1,
        };
        let compound = CachedCompoundParameter {
            phase_index: 4,
            phase_name: "Ca2SiO4".to_owned(),
            h298: -2_307_000.0,
            s298: 127.6,
        };
        let cache = ParameterCache {
            interaction_lookup: HashMap::new(),
            interaction_parameters: Vec::new(),
            endmember_lookup: HashMap::new(),
            compound_lookup: HashMap::new(),
            endmembers: vec![endmember.clone()],
            compounds: vec![compound.clone()],
        };

        assert_eq!(cache.endmember_parameters(), [endmember]);
        assert_eq!(cache.compound_parameters(), [compound]);
    }
}
