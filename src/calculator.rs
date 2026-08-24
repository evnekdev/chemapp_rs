// chemapp::calculator.rs

//! High-level, fallible ChemApp loading, calculation, mapping, and reporting.
//!
//! [`Calculator`] is the recommended entry point for most applications. It
//! owns one live [`Engine`], loads one thermodynamic system, and
//! can transform user composition bases through `chemformula`. ChemApp remains
//! stateful: calculations, conditions, mappings, and parameter changes all act
//! on that one native engine instance.
use chemformula::Transform;
use nalgebra::{DVector, Dim, Storage, Vector};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use tempfile::NamedTempFile;

use crate::cache::ParameterCache;
use crate::entities::system::System;
use crate::interactions::{
    InteractionChannel, InteractionDescriptorCrossCheck, InteractionParameter,
    InteractionParameterAddress, PhaseInteractionReport,
};
use crate::iterator::PhaseIterator;
use crate::iterator::SystemComponentIterator;
use crate::snapshot::{CalculatorSnapshot, SnapshotOptions};
use crate::{error::ChemAppError, Engine};

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

/// ChemApp's three thermochemical data-file formats supported by the loader.
///
/// This is deliberately determined before querying or mutating native state so
/// an unsupported filename cannot open a native FORTRAN unit accidentally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DatafileFormat {
    Ascii,
    Binary,
    Transparent,
}

/// Executes the native FIRST/NEXT continuation protocol and captures every
/// successful result before the next state transition.
fn collect_mapping_results<T, C, S>(
    first_option: &str,
    next_option: &str,
    mut call: C,
    mut snapshot: S,
) -> Result<Vec<T>, ChemAppError>
where
    C: FnMut(&str) -> Result<i32, ChemAppError>,
    S: FnMut() -> Result<T, ChemAppError>,
{
    let mut results = Vec::new();
    let mut continuation = call(first_option)?;
    results.push(snapshot()?);
    while continuation > 0 {
        continuation = call(next_option)?;
        results.push(snapshot()?);
    }
    Ok(results)
}

/// Determines the ChemApp data-file format from its case-insensitive extension.
fn datafile_format_from_filename(filename: &str) -> Result<DatafileFormat, ChemAppError> {
    let extension = Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| ChemAppError::OtherError(format!("{filename} has no extension")))?;

    match extension.as_str() {
        "dat" => Ok(DatafileFormat::Ascii),
        "bin" => Ok(DatafileFormat::Binary),
        "cst" => Ok(DatafileFormat::Transparent),
        _ => Err(ChemAppError::OtherError(format!(
            "{extension} is not a recognized datafile extension for {filename}"
        ))),
    }
}

/// Executes two complete TQCLIM orderings at most once each. A failed first
/// write may already have mutated ChemApp's target-limit state, so the retry
/// deliberately writes *both* bounds again rather than retrying only the call
/// which reported the error.
fn set_temperature_limits_with_retry<F>(
    interval: (f64, f64),
    inverse_order: bool,
    mut set_limit: F,
) -> Result<(), ChemAppError>
where
    F: FnMut(&'static str, f64) -> Result<(), ChemAppError>,
{
    let low = ("TLOW", interval.0);
    let high = ("THIGH", interval.1);
    let (preferred, alternate) = if inverse_order {
        ([high, low], [low, high])
    } else {
        ([low, high], [high, low])
    };

    let attempt = |ordering: [(&'static str, f64); 2], setter: &mut F| {
        setter(ordering[0].0, ordering[0].1)?;
        setter(ordering[1].0, ordering[1].1)
    };

    match attempt(preferred, &mut set_limit) {
        Ok(()) => Ok(()),
        Err(preferred_error) => match attempt(alternate, &mut set_limit) {
            Ok(()) => Ok(()),
            Err(alternate_error) => Err(ChemAppError::RetryError {
                operation: "setting ChemApp temperature target limits".to_owned(),
                preferred: Box::new(preferred_error),
                alternate: Box::new(alternate_error),
            }),
        },
    }
}

/// Builds a `chemformula` transform without allowing its linear-solver panic
/// to escape a fallible public `Calculator` API.
fn build_transform<T1: AsRef<str>, T2: AsRef<str>>(
    initial: &[T1],
    final_basis: &[T2],
) -> Result<Transform, ChemAppError> {
    match catch_unwind(AssertUnwindSafe(|| Transform::new(initial, final_basis, true))) {
        Ok(Ok(transform)) => Ok(transform),
        Ok(Err(error)) => Err(ChemAppError::OtherError(format!(
            "could not construct the ChemApp composition transform: {error:?}"
        ))),
        Err(_) => Err(ChemAppError::OtherError(
            "could not construct the ChemApp composition transform: the selected basis does not span the loaded system-component basis"
                .to_owned(),
        )),
    }
}

fn validate_composition_rows(actual: usize, expected: usize) -> Result<(), ChemAppError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ChemAppError::OtherError(format!(
            "composition has {actual} entries, but the active input basis requires {expected}"
        )))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TargetRatioConstraint {
    master_phase: usize,
    fixed_component: usize,
    adjusting_component: usize,
}

/// Maximum number of expensive target-temperature equilibria in one ratio solve.
const TARGET_RATIO_MAX_EVALUATIONS: usize = 32;
/// A one-part-per-million mismatch in the logarithmic incoming/phase ratio.
/// This is materially tighter than ordinary thermochemical repeatability while
/// avoiding a claim of machine-precision convergence from the native solver.
const TARGET_RATIO_RESIDUAL_TOLERANCE: f64 = 1.0e-6;
/// Before a bracket exists, one trial may change the ratio by at most a factor
/// of 100 in either direction. Bracketed steps remain inside their bracket.
const TARGET_RATIO_MAX_LOG_STEP: f64 = 4.605_170_185_988_092;
const TARGET_RATIO_SECANT_DENOMINATOR_TOLERANCE: f64 = 1.0e-12;
const TARGET_RATIO_BRACKET_GUARD_FRACTION: f64 = 0.1;
const TARGET_RATIO_MIN_EXPLORATION_STEP: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
struct TargetRatioSettings {
    max_evaluations: usize,
    residual_tolerance: f64,
    max_log_step: f64,
}

impl Default for TargetRatioSettings {
    fn default() -> Self {
        Self {
            max_evaluations: TARGET_RATIO_MAX_EVALUATIONS,
            residual_tolerance: TARGET_RATIO_RESIDUAL_TOLERANCE,
            max_log_step: TARGET_RATIO_MAX_LOG_STEP,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TargetRatioTrial {
    fixed_fraction: f64,
    adjusting_fraction: f64,
}

#[derive(Clone, Copy, Debug)]
struct TargetRatioPoint {
    log_ratio: f64,
    log_phase_ratio: f64,
    ratio: f64,
    phase_ratio: f64,
    residual: f64,
}

#[derive(Clone, Copy, Debug)]
enum TargetRatioEvaluation {
    ZeroBoundary,
    PositiveSeed(f64),
    ZeroPhaseAtPositiveInput,
    Positive(TargetRatioPoint),
}

#[derive(Clone, Copy, Debug)]
struct TargetRatioSolution {
    ratio: f64,
    phase_ratio: f64,
    residual: f64,
    evaluations: usize,
    bracketed: bool,
    used_secant: bool,
}

impl TargetRatioSolution {
    /// Guards the solver's internal success invariant at its production caller.
    fn confirm(self, tolerance: f64) -> Result<(), ChemAppError> {
        if self.ratio.is_finite()
            && self.ratio >= 0.0
            && self.phase_ratio.is_finite()
            && self.phase_ratio >= 0.0
            && self.residual.is_finite()
            && self.residual.abs() <= tolerance
            && self.evaluations > 0
        {
            Ok(())
        } else {
            Err(ChemAppError::OtherError(format!(
                "calculate_target_t solver returned an invalid success state: ratio={}, phase ratio={}, residual={}, evaluations={}, bracketed={}, used secant={}",
                self.ratio,
                self.phase_ratio,
                self.residual,
                self.evaluations,
                self.bracketed,
                self.used_secant
            )))
        }
    }
}

fn target_ratio_constraint(
    composition_len: usize,
    master_phase: usize,
    fixed: Option<usize>,
    adjusting: Option<usize>,
) -> Result<Option<TargetRatioConstraint>, ChemAppError> {
    match (fixed, adjusting) {
        (None, None) => Ok(None),
        (Some(fixed), Some(adjusting)) => {
            for (role, index) in [("fixed", fixed), ("adjusting", adjusting)] {
                if index == 0 || index > composition_len {
                    return Err(ChemAppError::OtherError(format!(
                        "{role} system-component index {index} is outside the one-based range 1..={composition_len}"
                    )));
                }
            }
            if fixed == adjusting {
                return Err(ChemAppError::OtherError(
                    "fixed and adjusting system-component indices must be different".to_owned(),
                ));
            }
            Ok(Some(TargetRatioConstraint {
                master_phase,
                fixed_component: fixed,
                adjusting_component: adjusting,
            }))
        }
        _ => Err(ChemAppError::OtherError(
            "both fixed and adjusting system components must be defined, or neither".to_owned(),
        )),
    }
}

fn target_ratio_residual(ratio: f64, phase_ratio: f64) -> Result<f64, ChemAppError> {
    if !ratio.is_finite() || !phase_ratio.is_finite() || ratio <= 0.0 || phase_ratio <= 0.0 {
        return Err(ChemAppError::OtherError(
            "target composition ratio residual requires finite, strictly positive ratios"
                .to_owned(),
        ));
    }
    let residual = ratio.ln() - phase_ratio.ln();
    if !residual.is_finite() {
        return Err(ChemAppError::OtherError(
            "target composition ratio residual is not finite".to_owned(),
        ));
    }
    Ok(residual)
}

fn initial_target_ratio(fixed_amount: f64, adjusting_amount: f64) -> Result<f64, ChemAppError> {
    if !fixed_amount.is_finite() || fixed_amount <= 0.0 {
        return Err(ChemAppError::OtherError(
            "calculate_target_t requires a finite, positive incoming fixed-component amount"
                .to_owned(),
        ));
    }
    if !adjusting_amount.is_finite() || adjusting_amount < 0.0 {
        return Err(ChemAppError::OtherError(
            "calculate_target_t requires a finite, non-negative incoming adjusting-component amount"
                .to_owned(),
        ));
    }
    let ratio = adjusting_amount / fixed_amount;
    if !ratio.is_finite() {
        return Err(ChemAppError::OtherError(
            "calculate_target_t initial incoming ratio is not finite".to_owned(),
        ));
    }
    Ok(ratio)
}

fn target_ratio_point(
    ratio: f64,
    trial: TargetRatioTrial,
) -> Result<TargetRatioEvaluation, ChemAppError> {
    if !ratio.is_finite() || ratio < 0.0 {
        return Err(ChemAppError::OtherError(
            "target composition correction produced an invalid incoming ratio".to_owned(),
        ));
    }
    if !trial.fixed_fraction.is_finite() || !trial.adjusting_fraction.is_finite() {
        return Err(ChemAppError::OtherError(
            "target composition correction received a non-finite master-phase fraction".to_owned(),
        ));
    }
    if trial.fixed_fraction <= 0.0 {
        return Err(ChemAppError::OtherError(
            "target composition correction requires a positive fixed-component master-phase fraction"
                .to_owned(),
        ));
    }
    if trial.adjusting_fraction < 0.0 {
        return Err(ChemAppError::OtherError(
            "target composition correction received a negative adjusting-component master-phase fraction"
                .to_owned(),
        ));
    }
    if trial.adjusting_fraction == 0.0 {
        return Ok(if ratio == 0.0 {
            TargetRatioEvaluation::ZeroBoundary
        } else {
            TargetRatioEvaluation::ZeroPhaseAtPositiveInput
        });
    }
    let phase_ratio = trial.adjusting_fraction / trial.fixed_fraction;
    if !phase_ratio.is_finite() || phase_ratio <= 0.0 {
        return Err(ChemAppError::OtherError(
            "target composition correction produced an invalid master-phase ratio".to_owned(),
        ));
    }
    if ratio == 0.0 {
        return Ok(TargetRatioEvaluation::PositiveSeed(phase_ratio));
    }
    let log_ratio = ratio.ln();
    let log_phase_ratio = phase_ratio.ln();
    let residual = target_ratio_residual(ratio, phase_ratio)?;
    if !log_ratio.is_finite() || !log_phase_ratio.is_finite() {
        return Err(ChemAppError::OtherError(
            "target composition correction produced a non-finite logarithmic ratio".to_owned(),
        ));
    }
    Ok(TargetRatioEvaluation::Positive(TargetRatioPoint {
        log_ratio,
        log_phase_ratio,
        ratio,
        phase_ratio,
        residual,
    }))
}

fn opposite_sign(left: f64, right: f64) -> bool {
    left.is_sign_positive() != right.is_sign_positive()
}

fn bounded_log_step(origin: f64, proposal: f64, maximum: f64) -> f64 {
    proposal.clamp(origin - maximum, origin + maximum)
}

fn find_narrowest_bracket(
    points: &[TargetRatioPoint],
) -> Option<(TargetRatioPoint, TargetRatioPoint)> {
    let mut bracket: Option<(TargetRatioPoint, TargetRatioPoint)> = None;
    for (offset, left) in points.iter().enumerate() {
        for right in &points[offset + 1..] {
            if opposite_sign(left.residual, right.residual) {
                let candidate = if left.log_ratio <= right.log_ratio {
                    (*left, *right)
                } else {
                    (*right, *left)
                };
                if bracket.is_none_or(|current| {
                    candidate.1.log_ratio - candidate.0.log_ratio
                        < current.1.log_ratio - current.0.log_ratio
                }) {
                    bracket = Some(candidate);
                }
            }
        }
    }
    bracket
}

fn target_ratio_nonconvergence(
    constraint: TargetRatioConstraint,
    evaluations: usize,
    best: Option<TargetRatioPoint>,
) -> ChemAppError {
    let detail = best.map_or_else(
        || "best residual unavailable".to_owned(),
        |point| {
            format!(
                "best residual={:.6e}, best incoming ratio={:.6e}, best phase ratio={:.6e}",
                point.residual.abs(),
                point.ratio,
                point.phase_ratio
            )
        },
    );
    ChemAppError::OtherError(format!(
        "calculate_target_t composition-ratio solve did not converge after {evaluations} equilibrium evaluations ({detail}; master phase={}, fixed component={}, adjusting component={})",
        constraint.master_phase, constraint.fixed_component, constraint.adjusting_component
    ))
}

/// Drives the expensive native target calculation through a pure, derivative-
/// free scalar state machine. The evaluator owns the one-TQCE-per-trial rule.
fn solve_target_composition_ratio<F>(
    initial_ratio: f64,
    constraint: TargetRatioConstraint,
    settings: TargetRatioSettings,
    mut evaluate: F,
) -> Result<TargetRatioSolution, ChemAppError>
where
    F: FnMut(f64) -> Result<TargetRatioTrial, ChemAppError>,
{
    if !initial_ratio.is_finite() || initial_ratio < 0.0 {
        return Err(ChemAppError::OtherError(
            "calculate_target_t requires a finite, non-negative initial adjusting/fixed ratio"
                .to_owned(),
        ));
    }
    if settings.max_evaluations == 0
        || !settings.residual_tolerance.is_finite()
        || settings.residual_tolerance <= 0.0
        || !settings.max_log_step.is_finite()
        || settings.max_log_step <= 0.0
    {
        return Err(ChemAppError::OtherError(
            "calculate_target_t received invalid private solver settings".to_owned(),
        ));
    }

    let mut next_ratio = initial_ratio;
    let mut points = Vec::new();
    let mut best: Option<TargetRatioPoint> = None;
    let mut bracketed = false;
    let mut used_secant = false;
    let mut exploration_step = TARGET_RATIO_MIN_EXPLORATION_STEP;

    for evaluations in 1..=settings.max_evaluations {
        let trial = evaluate(next_ratio)?;
        let evaluation = target_ratio_point(next_ratio, trial).map_err(|error| {
            ChemAppError::OtherError(format!(
                "calculate_target_t could not construct residual at equilibrium evaluation {evaluations} (master phase={}, fixed component={}, adjusting component={}): {}",
                constraint.master_phase,
                constraint.fixed_component,
                constraint.adjusting_component,
                error.description()
            ))
        })?;
        let point = match evaluation {
            // Exact zero is a physical boundary, not an epsilon-substituted log value.
            TargetRatioEvaluation::ZeroBoundary => {
                return Ok(TargetRatioSolution {
                    ratio: 0.0,
                    phase_ratio: 0.0,
                    residual: 0.0,
                    evaluations,
                    bracketed,
                    used_secant,
                });
            }
            TargetRatioEvaluation::PositiveSeed(phase_ratio) => {
                next_ratio = phase_ratio;
                continue;
            }
            // Positive incoming amount with zero phase fraction explicitly
            // probes the exact zero boundary on the next expensive trial.
            TargetRatioEvaluation::ZeroPhaseAtPositiveInput => {
                next_ratio = 0.0;
                continue;
            }
            TargetRatioEvaluation::Positive(point) => point,
        };

        if point.residual.abs() <= settings.residual_tolerance {
            return Ok(TargetRatioSolution {
                ratio: point.ratio,
                phase_ratio: point.phase_ratio,
                residual: point.residual,
                evaluations,
                bracketed,
                used_secant,
            });
        }
        if best.is_none_or(|current| point.residual.abs() < current.residual.abs()) {
            best = Some(point);
        }
        points.push(point);

        if let Some((left, right)) = find_narrowest_bracket(&points) {
            bracketed = true;
            let width = right.log_ratio - left.log_ratio;
            let denominator = right.residual - left.residual;
            let secant =
                right.log_ratio - right.residual * (right.log_ratio - left.log_ratio) / denominator;
            let guard = width * TARGET_RATIO_BRACKET_GUARD_FRACTION;
            let next_log_ratio = if denominator.is_finite()
                && denominator.abs() > TARGET_RATIO_SECANT_DENOMINATOR_TOLERANCE
                && secant.is_finite()
                && secant > left.log_ratio + guard
                && secant < right.log_ratio - guard
            {
                used_secant = true;
                secant
            } else {
                left.log_ratio + width * 0.5
            };
            next_ratio = next_log_ratio.exp();
        } else {
            let best_point = best.ok_or_else(|| {
                ChemAppError::OtherError(
                    "calculate_target_t solver lost its just-evaluated finite point".to_owned(),
                )
            })?;
            let proposal = if points.len() == 1 {
                // The native phase ratio is the physically meaningful first
                // predictor, equivalent to one legacy Picard update.
                best_point.log_phase_ratio
            } else {
                let previous = points[points.len() - 2];
                let current = points[points.len() - 1];
                let denominator = current.residual - previous.residual;
                let secant = current.log_ratio
                    - current.residual * (current.log_ratio - previous.log_ratio) / denominator;
                if denominator.is_finite()
                    && denominator.abs() > TARGET_RATIO_SECANT_DENOMINATOR_TOLERANCE
                    && secant.is_finite()
                {
                    used_secant = true;
                    secant
                } else {
                    best_point.log_phase_ratio
                }
            };
            let mut next_log_ratio =
                bounded_log_step(best_point.log_ratio, proposal, settings.max_log_step);
            if !next_log_ratio.is_finite()
                || points
                    .iter()
                    .any(|known| (known.log_ratio - next_log_ratio).abs() <= f64::EPSILON * 16.0)
            {
                let direction = if best_point.residual.is_sign_positive() {
                    -1.0
                } else {
                    1.0
                };
                next_log_ratio =
                    best_point.log_ratio + direction * exploration_step.min(settings.max_log_step);
                exploration_step = (exploration_step * 2.0).min(settings.max_log_step);
            }
            next_ratio = next_log_ratio.exp();
        }

        if !next_ratio.is_finite() || next_ratio <= 0.0 {
            return Err(ChemAppError::OtherError(format!(
                "calculate_target_t produced an invalid next incoming ratio for master phase={}, fixed component={}, adjusting component={}",
                constraint.master_phase, constraint.fixed_component, constraint.adjusting_component
            )));
        }
    }

    Err(target_ratio_nonconvergence(
        constraint,
        settings.max_evaluations,
        best,
    ))
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

/// A high-level owner of one loaded, stateful ChemApp engine.
///
/// Construction is deliberately fallible because the native library, licence,
/// architecture, and thermodynamic data-file are external requirements. The
/// type does not implement [`Default`]; use [`Calculator::from_library`] or
/// [`Calculator::from_library_unloaded`] so loading failures are reported.
///
/// A calculator owns one mutable ChemApp state. It is not safe to share one
/// calculator for concurrent native calls; parallel calculations require
/// separately loaded library instances/copies supported by the installation.
/// Its Engine cannot be replaced independently of the loaded-system metadata:
/// use [`Calculator::engine`] for deliberate native calls.
///
/// ```compile_fail
/// use chemapp_rs::{Calculator, Engine};
///
/// fn replace_engine(calculator: &mut Calculator, replacement: Engine) {
///     calculator.engine = replacement;
/// }
/// ```
///
/// ```compile_fail
/// use chemapp_rs::Calculator;
/// fn assert_send<T: Send>() {}
/// assert_send::<Calculator>();
/// ```
///
/// ```compile_fail
/// use chemapp_rs::Calculator;
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<Calculator>();
/// ```
#[derive(Debug)]
pub struct Calculator {
    /// The loaded low-level ChemApp engine. Keeping ownership private prevents
    /// replacement without rebuilding all system-dependent metadata.
    engine: Engine,
    /// Optional captured baselines for reversible parameter changes.
    cache: Option<ParameterCache>,
    /// Data-file path used to initialize this calculator, or empty if unloaded.
    file: String,
    /// Active temporary error-file path and FORTRAN unit, when redirected.
    nondefault_errunit: Option<(String, usize)>,
    /// Composition transform from the user-selected basis to system components.
    transform: Transform,
    /// The ERROR unit active before `redirect_error_to_temp` changed it.
    previous_errunit: Option<usize>,
    /// Names leased to live high-level `Stream` owners. ChemApp streams are
    /// name-addressed, so allowing duplicate owners would make `Drop` remove
    /// a stream still represented by another Rust value.
    pub(crate) active_stream_names: RefCell<HashSet<String>>,
}

impl Calculator {
    /// Returns the underlying stateful low-level ChemApp engine.
    ///
    /// This is the advanced native escape hatch. Ordinary result queries and
    /// deliberate low-level state operations are supported, but raw calls that
    /// reinitialize or replace the loaded thermodynamic system (for example
    /// `TQINI` followed by a data-file read) invalidate this Calculator's
    /// stored data-file identity, composition transform, parameter cache, and
    /// other system-local high-level identities. Construct a new Calculator
    /// instead of using such a sequence through this accessor.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Read one typed interaction parameter from the current live TQGPAR matrix.
    ///
    /// `address` is valid only for this Calculator's loaded thermodynamic
    /// system and configuration; it is not a persistent cross-system identity.
    pub fn interaction_parameter(
        &self,
        address: InteractionParameterAddress,
    ) -> Result<InteractionParameter, ChemAppError> {
        crate::interactions::read_interaction_parameter(&self.engine, address)
    }

    /// Change one verified interaction parameter through its exact TQCDAT address.
    ///
    /// This mutates the loaded ChemApp model in memory. It deliberately does
    /// not reset conditions or recalculate equilibrium, so previously obtained
    /// results are stale until the caller explicitly calculates again. The
    /// address must have been obtained for this Calculator's current loaded
    /// system and configuration.
    pub fn set_interaction_parameter(
        &self,
        address: InteractionParameterAddress,
        value: f64,
    ) -> Result<(), ChemAppError> {
        crate::interactions::validate_interaction_parameter_mutation(&self.engine, address)?;
        crate::interactions::write_interaction_parameter(&self.engine, address, value)
    }

    /// Queries the loaded system's component names without turning a failed
    /// native lookup into a silently incomplete composition basis.
    fn component_names(engine: &Engine) -> Result<Vec<String>, ChemAppError> {
        let count = engine.tqnosc()?;
        (0..count).map(|index| engine.tqgnsc(index + 1)).collect()
    }

    /// Builds a formula transform while retaining the dependency's diagnostic.
    fn identity_transform(components: &[String]) -> Result<Transform, ChemAppError> {
        build_transform(components, components)
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Initialize a [`Calculator`] from a ChemApp dll file and a datafile
    pub fn from_library(libname: &str, datfile: &str) -> Result<Calculator, ChemAppError> {
        let engine = Engine::new(libname)?;
        Self::init_engine(&engine, datfile)?;
        let components = Self::component_names(&engine)?;
        let transform = Self::identity_transform(&components)?;
        Ok(Calculator {
            engine,
            cache: None,
            file: datfile.to_string(),
            nondefault_errunit: None,
            transform,
            previous_errunit: None,
            active_stream_names: RefCell::new(HashSet::new()),
        })
    }

    /// Initializes ChemApp without loading a thermodynamic data-file.
    ///
    /// This constructor supports version/interface/licensing queries and other
    /// native operations whose ChemApp contract does not require a loaded
    /// system. System-dependent high-level operations require
    /// [`Calculator::from_library`] unless their own documentation explicitly
    /// states otherwise.
    ///
    /// This is not a high-level reload workflow. Loading a data-file manually
    /// through [`Calculator::engine`] does not update this Calculator's stored
    /// data-file identity, composition transform, parameter cache, or other
    /// system-local metadata. Construct a new loaded Calculator instead.
    pub fn from_library_unloaded(libname: &str) -> Result<Calculator, ChemAppError> {
        let engine = Engine::new(libname)?;
        engine.tqini()?;
        Ok(Calculator {
            engine,
            cache: None,
            file: "".to_string(),
            nondefault_errunit: None,
            transform: Transform::default(),
            previous_errunit: None,
            active_stream_names: RefCell::new(HashSet::new()),
        })
    }

    /// Claims the unique high-level owner for a name-addressed native stream.
    /// The ChemApp manual specifies stream creation/removal by identifier but
    /// does not define duplicate-definition semantics, so this wrapper avoids
    /// relying on an undocumented replace/share behavior.
    pub(crate) fn claim_stream_name(&self, name: &str) -> Result<(), ChemAppError> {
        let mut names = self.active_stream_names.borrow_mut();
        if names.insert(name.to_owned()) {
            Ok(())
        } else {
            Err(ChemAppError::OtherError(format!(
                "a live Stream already owns the ChemApp stream name {name:?}"
            )))
        }
    }

    pub(crate) fn release_stream_name(&self, name: &str) {
        self.active_stream_names.borrow_mut().remove(name);
    }

    /// Returns the thermodynamic data-file used to load this calculator.
    pub fn datafile(&self) -> Option<&str> {
        (!self.file.is_empty()).then_some(self.file.as_str())
    }

    /// Returns the active user-basis-to-system-component transform.
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Returns the captured parameter cache, if one has been generated.
    ///
    /// The returned cache is an inspection view. Mutation and reset operations
    /// live on `Calculator` so the cache can only target its owning Engine.
    pub fn parameter_cache(&self) -> Option<&ParameterCache> {
        self.cache.as_ref()
    }

    pub(crate) fn install_parameter_cache(&mut self, cache: ParameterCache) {
        self.cache = Some(cache);
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Initializes the ChemApp interface and preconfigures it with the thermodynamic info from a datafile.
    fn init_engine(engine: &Engine, datfile: &str) -> Result<(), ChemAppError> {
        engine.tqini()?;
        Self::load_datafile(engine, datfile)?;
        Ok(())
    }

    /// Loads one ChemApp thermochemical data-file through its format-specific
    /// native open/read/close sequence.
    ///
    /// The extension is validated before native state is queried. The configured
    /// `FILE` unit from `TQGIO` is then used consistently for opening and
    /// closing. Once an open succeeds, close is attempted even if the read
    /// fails; a dual failure retains the read error as the primary error.
    fn load_datafile(engine: &Engine, datfile: &str) -> Result<(), ChemAppError> {
        let format = datafile_format_from_filename(datfile)?;
        let unit = engine.tqgio("FILE")?;

        match format {
            DatafileFormat::Ascii => engine.tqopna(datfile, unit)?,
            DatafileFormat::Binary => engine.tqopnb(datfile, unit)?,
            DatafileFormat::Transparent => engine.tqopnt(datfile, unit)?,
        }

        let read_result = match format {
            DatafileFormat::Ascii => engine.tqrfil(),
            DatafileFormat::Binary => engine.tqrbin(),
            DatafileFormat::Transparent => engine.tqrcst(),
        };
        let close_result = engine.tqclos(unit);

        match (read_result, close_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(close)) => Err(close),
            (Err(read), Ok(())) => Err(read),
            (Err(read), Err(close)) => Err(ChemAppError::CleanupError {
                operation: format!("reading {datfile} through ChemApp unit {unit}"),
                primary: Box::new(read),
                cleanup: Box::new(close),
            }),
        }
    }
    /// Sets a formula transform for input compositions.
    ///
    /// The transform defines the active high-level component basis. Changing
    /// it drops any captured parameter cache so system-local cached addresses
    /// cannot outlive the basis/configuration that produced them.
    pub fn set_transform<T: AsRef<str>>(&mut self, basis: &[T]) -> Result<(), ChemAppError> {
        let components = Self::component_names(&self.engine)?;
        self.transform = build_transform(&components, basis)?;
        // Cached native addresses are system-local. A basis change does not
        // alter ChemApp's native indices today, but invalidating here keeps the
        // high-level ownership contract simple and future-proof.
        self.cache = None;
        Ok(())
    }
    /// Internally, creates a temporary file (deleted once the current `Calculator` instance is dropped) to redirect ChemApp outputs; this is a useful feature in environments where console window is not available.
    pub fn redirect_error_to_temp(&mut self) -> Result<(), ChemAppError> {
        let directory = Path::new(&self.engine.library_name)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        let temp_file = NamedTempFile::new_in(&directory).map_err(|error| {
            ChemAppError::OtherError(format!(
                "could not create a temporary ChemApp ERROR file in {}: {error}",
                directory.display()
            ))
        })?;
        let (_file, temp_path) = temp_file.keep().map_err(|error| {
            ChemAppError::OtherError(format!(
                "could not persist the temporary ChemApp ERROR file: {}",
                error.error
            ))
        })?;
        let filename = match temp_path.to_str() {
            Some(path) => path.to_owned(),
            None => {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ChemAppError::OtherError(
                    "temporary ChemApp ERROR path is not valid UTF-8".to_owned(),
                ));
            }
        };
        let unit = 30;
        let previous_unit = match self.engine.tqgio("ERROR") {
            Ok(unit) => unit,
            Err(error) => {
                let _ = std::fs::remove_file(&temp_path);
                return Err(error);
            }
        };
        if let Err(open) = self.engine.tqopen(&filename, unit) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(open);
        }
        if let Err(configure) = self.engine.tqcio("ERROR", unit) {
            let close = self.engine.tqclos(unit);
            let _ = std::fs::remove_file(&temp_path);
            return match close {
                Ok(()) => Err(configure),
                Err(cleanup) => Err(ChemAppError::CleanupError {
                    operation: "redirecting ChemApp ERROR output".to_owned(),
                    primary: Box::new(configure),
                    cleanup: Box::new(cleanup),
                }),
            };
        }
        self.nondefault_errunit = Some((filename, unit));
        self.previous_errunit = Some(previous_unit);
        Ok(())
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Resets all input conditions to prepare for another calculation (with the same datafile).
    pub fn reset(&self) -> Result<(), ChemAppError> {
        self.engine.tqremc(-2)?;
        Ok(())
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Global system properties accessor.
    pub fn system(&self) -> System<'_> {
        System::new(self)
    }

    /// Iterates over system component indices.
    pub fn components(&self) -> Result<SystemComponentIterator<'_>, ChemAppError> {
        SystemComponentIterator::new(self)
    }
    /// Iterates over phase indices.
    pub fn phases(&self) -> Result<PhaseIterator<'_>, ChemAppError> {
        PhaseIterator::new(self)
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Formats the live calculated state as a multi-section report.
    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_report(self)
    }

    /// Prints the current whole-system table to standard output.
    pub fn print_system(&self) -> Result<(), ChemAppError> {
        println!("{}", self.system().table_string()?);
        Ok(())
    }

    /// Prints one table for every system component.
    pub fn print_components(&self) -> Result<(), ChemAppError> {
        for component in self.components()? {
            println!("{}", component.table_string()?);
        }
        Ok(())
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Sets temperature target limits with at most two complete orderings.
    ///
    /// If either call in the preferred ordering fails, this writes both bounds
    /// again in reverse order. A failed call may already have changed native
    /// target-limit state; this bounded protocol makes no reset claim.
    pub fn set_clim(&self, interval: (f64, f64), inverse_order: bool) -> Result<(), ChemAppError> {
        set_temperature_limits_with_retry(interval, inverse_order, |option, value| {
            self.engine.tqclim(option, value)
        })
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    fn transformed_composition<D: Dim, S: Storage<f64, D>>(
        &self,
        compositions: &Vector<f64, D, S>,
    ) -> Result<DVector<f64>, ChemAppError> {
        let expected = self.transform.number_final();
        validate_composition_rows(compositions.nrows(), expected)?;
        Ok(self
            .transform
            .transform_final2init(compositions, false, false, false)
            .column(0)
            .into_owned())
    }

    /// Executes the shared no-target protocol after composition conversion.
    fn calculate_isothermal_(
        &self,
        x_i: &DVector<f64>,
        temp: f64,
        pressure: Option<f64>,
    ) -> Result<(), ChemAppError> {
        self.reset()?;
        if let Some(pressure) = pressure {
            self.engine.tqsetc("P", 0, 0, pressure)?;
        }
        self.engine.tqsetc("T", 0, 0, temp)?;
        for k in 0..x_i.len() {
            self.engine.tqsetc("IA", 0, k + 1, x_i[k])?;
        }
        //self.engine.tqshow();
        self.engine.tqce(" ", 0, 0, (0.0, 0.0))?;
        Ok(())
    }

    /// Calculates a no-target equilibrium at `temp` and ChemApp's default pressure.
    ///
    /// The call removes all prior conditions with `TQREMC(-2)`, converts the
    /// supplied mole amounts from the active user basis into the loaded system-
    /// component basis, sets `T` and each `IA`, then calls `TQCE` with a blank
    /// target. The documented default pressure after reset is 1 bar. The
    /// calculated equilibrium remains the live native state on return.
    pub fn calculate_isothermal<D: Dim, S: Storage<f64, D>>(
        &self,
        compositions: &Vector<f64, D, S>,
        temp: f64,
    ) -> Result<(), ChemAppError> {
        self.calculate_isothermal_(&self.transformed_composition(compositions)?, temp, None)
    }

    /// Calculates a no-target equilibrium at explicit temperature and pressure.
    ///
    /// Temperature and pressure use the engine's active units. Reset,
    /// composition conversion, amount basis, and live-state semantics are the
    /// same as [`Calculator::calculate_isothermal`].
    pub fn calculate_isothermal_at_pressure<D: Dim, S: Storage<f64, D>>(
        &self,
        compositions: &Vector<f64, D, S>,
        temp: f64,
        pressure: f64,
    ) -> Result<(), ChemAppError> {
        self.calculate_isothermal_(
            &self.transformed_composition(compositions)?,
            temp,
            Some(pressure),
        )
    }

    /// Evaluates one incoming-ratio trial with exactly one target TQCE call.
    /// The fixed amount and every unrelated component remain unchanged; only
    /// the adjusting component is replaced at the native-call boundary.
    fn evaluate_target_t_trial(
        &self,
        base_composition: &DVector<f64>,
        constraint: TargetRatioConstraint,
        fixed_amount: f64,
        ratio: f64,
        interval: (f64, f64),
    ) -> Result<TargetRatioTrial, ChemAppError> {
        if !fixed_amount.is_finite() || fixed_amount <= 0.0 {
            return Err(ChemAppError::OtherError(
                "calculate_target_t requires a finite, positive incoming fixed-component amount"
                    .to_owned(),
            ));
        }
        if !ratio.is_finite() || ratio < 0.0 {
            return Err(ChemAppError::OtherError(
                "calculate_target_t trial requires a finite, non-negative incoming ratio"
                    .to_owned(),
            ));
        }
        let adjusting_amount = fixed_amount * ratio;
        if !adjusting_amount.is_finite() {
            return Err(ChemAppError::OtherError(
                "calculate_target_t trial produced a non-finite adjusting-component amount"
                    .to_owned(),
            ));
        }

        let mut trial_composition = base_composition.clone();
        trial_composition[constraint.adjusting_component - 1] = adjusting_amount;
        for (offset, amount) in trial_composition.iter().copied().enumerate() {
            if !amount.is_finite() {
                return Err(ChemAppError::OtherError(format!(
                    "calculate_target_t incoming amount for system component {} is not finite",
                    offset + 1
                )));
            }
            self.engine.tqsetc("IA", 0, offset + 1, amount)?;
        }
        self.engine.tqce("T", 0, 0, interval)?;
        Ok(TargetRatioTrial {
            fixed_fraction: self.engine.tqgetr(
                "XP",
                constraint.master_phase,
                constraint.fixed_component,
            )?,
            adjusting_fraction: self.engine.tqgetr(
                "XP",
                constraint.master_phase,
                constraint.adjusting_component,
            )?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn calculate_target_t_(
        &self,
        x_i: &DVector<f64>,
        masterphase: usize,
        target: usize,
        interval: (f64, f64),
        precipitation: bool,
        fixed: Option<usize>,
        adjusting: Option<usize>,
    ) -> Result<(), ChemAppError> {
        let constraint = target_ratio_constraint(x_i.len(), masterphase, fixed, adjusting)?;
        if target == 0 {
            return Err(ChemAppError::OtherError(
                "target phase index must be a positive one-based ChemApp index".to_owned(),
            ));
        }
        let correction = match constraint {
            Some(constraint) => {
                if constraint.master_phase == 0 {
                    return Err(ChemAppError::OtherError(
                        "master phase index must be a positive one-based ChemApp index".to_owned(),
                    ));
                }
                let fixed_amount = x_i[constraint.fixed_component - 1];
                let adjusting_amount = x_i[constraint.adjusting_component - 1];
                let ratio =
                    initial_target_ratio(fixed_amount, adjusting_amount).map_err(|error| {
                        ChemAppError::OtherError(format!(
                            "{} (fixed component={}, adjusting component={})",
                            error.description(),
                            constraint.fixed_component,
                            constraint.adjusting_component
                        ))
                    })?;
                Some((constraint, fixed_amount, ratio))
            }
            None => None,
        };
        // Set non-compositional conditions once. Trial evaluation changes only IA.
        let val = if precipitation { -0.5 } else { 0.0 };
        self.engine.tqsetc("A", target, 0, val)?;
        self.set_clim(interval, true)?;
        match correction {
            Some((constraint, fixed_amount, initial_ratio)) => {
                let solution = solve_target_composition_ratio(
                    initial_ratio,
                    constraint,
                    TargetRatioSettings::default(),
                    |ratio| {
                        self.evaluate_target_t_trial(x_i, constraint, fixed_amount, ratio, interval)
                    },
                )?;
                solution.confirm(TARGET_RATIO_RESIDUAL_TOLERANCE)?;
            }
            None => {
                for (offset, amount) in x_i.iter().copied().enumerate() {
                    if !amount.is_finite() {
                        return Err(ChemAppError::OtherError(format!(
                            "calculate_target_t incoming amount for system component {} is not finite",
                            offset + 1
                        )));
                    }
                    self.engine.tqsetc("IA", 0, offset + 1, amount)?;
                }
                self.engine.tqce("T", 0, 0, interval)?;
            }
        }
        Ok(())
    }

    /// Performs a temperature-target calculation after checked basis conversion.
    ///
    /// This method does **not** call `TQREMC`: it inherits the current pressure,
    /// units, phase/constituent statuses, and other active native conditions.
    /// It sets the phase-amount target `A`, rewrites temperature target limits,
    /// writes incoming component amounts, and calls `TQCE` with temperature as
    /// the target variable. `target` is a positive one-based phase index;
    /// `masterphase` is likewise required when composition correction is used.
    /// `fixed` and `adjusting` are optional, distinct,
    /// one-based system-component indices; supply both or neither. Supplying
    /// neither performs exactly one native target equilibrium.
    ///
    /// When both are supplied, the fixed incoming amount and all unrelated
    /// amounts remain constant while only the adjusting amount is changed. A
    /// safeguarded scalar solve in logarithmic ratio space requires the incoming
    /// ratio `IA_adjusting / IA_fixed` to agree with the resulting master-phase
    /// ratio `XP_adjusting / XP_fixed`. Exact zero-ratio boundaries are handled
    /// without inventing an epsilon. Exhausting the equilibrium-evaluation
    /// budget is an explicit non-convergence error, never success.
    ///
    /// On success, the target equilibrium and the conditions just described
    /// remain live. On failure, ChemApp may retain the last trial and any
    /// conditions written before the failing call; this method does not attempt
    /// a hidden rollback. Native trial errors propagate immediately.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_target_t<D: Dim, S: Storage<f64, D>>(
        &self,
        compositions: &Vector<f64, D, S>,
        masterphase: usize,
        target: usize,
        interval: (f64, f64),
        precipitation: bool,
        fixed: Option<usize>,
        adjusting: Option<usize>,
    ) -> Result<(), ChemAppError> {
        self.calculate_target_t_(
            &self.transformed_composition(compositions)?,
            masterphase,
            target,
            interval,
            precipitation,
            fixed,
            adjusting,
        )
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Create a snapshot of the current state.
    pub fn snapshot(&self) -> Result<CalculatorSnapshot, ChemAppError> {
        CalculatorSnapshot::new(self)
    }

    /// Creates a deep snapshot using explicit phase-retention options.
    pub fn snapshot_with_options(
        &self,
        options: SnapshotOptions,
    ) -> Result<CalculatorSnapshot, ChemAppError> {
        CalculatorSnapshot::new_with_options(self, options)
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Lists name-resolved Gibbs interactions through the typed interaction pipeline.
    #[deprecated(note = "use Phase::gibbs_interactions or Calculator::interaction_report")]
    pub fn interactions_ge_expanded(&self, indexp: usize) -> Result<Vec<String>, ChemAppError> {
        Ok(crate::interactions::load_phase_interactions(
            &self.engine,
            indexp,
            InteractionChannel::GibbsExcess,
        )?
        .into_iter()
        .map(|interaction| interaction.resolved_text())
        .collect())
    }

    /// Lists name-resolved magnetic interactions through the typed interaction pipeline.
    #[deprecated(note = "use Phase::magnetic_interactions or Calculator::interaction_report")]
    pub fn interactions_magn_expanded(&self, indexp: usize) -> Result<Vec<String>, ChemAppError> {
        Ok(crate::interactions::load_phase_interactions(
            &self.engine,
            indexp,
            InteractionChannel::Magnetic,
        )?
        .into_iter()
        .map(|interaction| interaction.resolved_text())
        .collect())
    }

    /// Compatibility view of resolved Gibbs rows.
    #[deprecated(note = "use Phase::gibbs_interactions")]
    pub fn interactions_ge_expanded_species(
        &self,
        indexp: usize,
    ) -> Result<Vec<Vec<String>>, ChemAppError> {
        Ok(crate::interactions::load_phase_interactions(
            &self.engine,
            indexp,
            InteractionChannel::GibbsExcess,
        )?
        .into_iter()
        .map(|interaction| vec![interaction.resolved_text()])
        .collect())
    }
    /// Compatibility view of resolved magnetic rows.
    #[deprecated(note = "use Phase::magnetic_interactions")]
    pub fn interactions_magn_expanded_species(
        &self,
        indexp: usize,
    ) -> Result<Vec<Vec<String>>, ChemAppError> {
        Ok(crate::interactions::load_phase_interactions(
            &self.engine,
            indexp,
            InteractionChannel::Magnetic,
        )?
        .into_iter()
        .map(|interaction| vec![interaction.resolved_text()])
        .collect())
    }

    /// Collect both interaction channels for every non-PURE phase in the
    /// loaded data-file. These static model parameters are intentionally not
    /// duplicated into equilibrium snapshots.
    pub fn interaction_report(&self) -> Result<Vec<PhaseInteractionReport>, ChemAppError> {
        self.interaction_report_with_optional_cross_check(None)
    }

    /// Collect both channels with independent ASCII-DAT structural evidence.
    /// Provider failures and unexplained differences never invalidate healthy
    /// native rows, and live TQGPAR values are never replaced.
    pub fn interaction_report_with_cross_check(
        &self,
        cross_check: &dyn InteractionDescriptorCrossCheck,
    ) -> Result<Vec<PhaseInteractionReport>, ChemAppError> {
        self.interaction_report_with_optional_cross_check(Some(cross_check))
    }

    /// Compatibility forwarding name for
    /// [`Self::interaction_report_with_cross_check`].
    #[deprecated(note = "use interaction_report_with_cross_check")]
    pub fn interaction_report_with_recovery(
        &self,
        cross_check: &dyn InteractionDescriptorCrossCheck,
    ) -> Result<Vec<PhaseInteractionReport>, ChemAppError> {
        self.interaction_report_with_cross_check(cross_check)
    }

    fn interaction_report_with_optional_cross_check(
        &self,
        cross_check: Option<&dyn InteractionDescriptorCrossCheck>,
    ) -> Result<Vec<PhaseInteractionReport>, ChemAppError> {
        let mut reports = Vec::new();
        for phase_index in 1..=self.engine.tqnop()? {
            if self.engine.tqmodl(phase_index)? != "PURE" {
                reports.push(
                    crate::interactions::load_phase_interaction_report_with_cross_check(
                        &self.engine,
                        phase_index,
                        cross_check,
                    )?,
                );
            }
        }
        Ok(reports)
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/
    /// Runs a complete stateful map and snapshots every successful native result
    /// before advancing ChemApp to the next point.
    #[allow(clippy::too_many_arguments)]
    fn mapping(
        &self,
        first_option: &str,
        next_option: &str,
        indexp: usize,
        indexc: usize,
        interval: (f64, f64),
        list: bool,
        options: SnapshotOptions,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        let call = |option: &str| {
            if list {
                self.engine.tqmapl(option, indexp, indexc, interval)
            } else {
                self.engine.tqmap(option, indexp, indexc, interval)
            }
        };
        collect_mapping_results(first_option, next_option, call, || {
            self.snapshot_with_options(options)
        })
    }

    /// Maps temperature and snapshots every successful state before advancing.
    ///
    /// `list` selects the listing variant of the native mapping routine.
    pub fn mapping_temperature(
        &self,
        tmin: f64,
        tmax: f64,
        list: bool,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping_temperature_with_options(tmin, tmax, list, SnapshotOptions::all())
    }

    /// Maps temperature with explicit snapshot phase-retention options.
    pub fn mapping_temperature_with_options(
        &self,
        tmin: f64,
        tmax: f64,
        list: bool,
        options: SnapshotOptions,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping("TF", "TN", 0, 0, (tmin, tmax), list, options)
    }

    /// Maps pressure and snapshots every successful state before advancing.
    ///
    /// `list` selects the listing variant of the native mapping routine.
    pub fn mapping_pressure(
        &self,
        pmin: f64,
        pmax: f64,
        list: bool,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping_pressure_with_options(pmin, pmax, list, SnapshotOptions::all())
    }

    /// Maps pressure with explicit snapshot phase-retention options.
    pub fn mapping_pressure_with_options(
        &self,
        pmin: f64,
        pmax: f64,
        list: bool,
        options: SnapshotOptions,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping("PF", "PN", 0, 0, (pmin, pmax), list, options)
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

/// Custom `Drop` re-implementation to ensure any temporary files are deleted.
impl Drop for Calculator {
    fn drop(&mut self) {
        if let Some((filename, unit)) = &self.nondefault_errunit {
            if let Some(previous_unit) = self.previous_errunit {
                let _ = self.engine.tqcio("ERROR", previous_unit);
            }
            let _ = self.engine.tqclos(*unit);
            let _ = std::fs::remove_file(filename);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::DVector;

    fn test_constraint() -> TargetRatioConstraint {
        TargetRatioConstraint {
            master_phase: 1,
            fixed_component: 1,
            adjusting_component: 2,
        }
    }

    fn test_settings(max_evaluations: usize) -> TargetRatioSettings {
        TargetRatioSettings {
            max_evaluations,
            ..TargetRatioSettings::default()
        }
    }

    fn trial_for_phase_ratio(phase_ratio: f64) -> TargetRatioTrial {
        if phase_ratio == 0.0 {
            TargetRatioTrial {
                fixed_fraction: 1.0,
                adjusting_fraction: 0.0,
            }
        } else {
            let fixed_fraction = 1.0 / (1.0 + phase_ratio);
            TargetRatioTrial {
                fixed_fraction,
                adjusting_fraction: phase_ratio * fixed_fraction,
            }
        }
    }

    #[test]
    fn transform_boundary_rejects_unspanned_basis_without_unwinding() {
        assert!(build_transform(&["Al", "O"], &["Al2O3"]).is_err());
    }

    #[test]
    fn transform_dimension_contract_is_reported_before_chemformula_asserts() {
        let transform = build_transform(&["Al", "O"], &["Al", "O"]).unwrap();
        let composition = DVector::from_vec(vec![1.0]);
        let error =
            validate_composition_rows(composition.nrows(), transform.number_final()).unwrap_err();
        assert!(error.description().contains("requires 2"));
    }

    #[test]
    fn target_component_indices_are_one_based_bounded_and_paired() {
        assert!(target_ratio_constraint(3, 1, None, None).unwrap().is_none());
        assert_eq!(
            target_ratio_constraint(3, 4, Some(1), Some(2)).unwrap(),
            Some(TargetRatioConstraint {
                master_phase: 4,
                fixed_component: 1,
                adjusting_component: 2,
            })
        );
        for invalid in [
            (Some(0), Some(1)),
            (Some(1), Some(0)),
            (Some(4), Some(1)),
            (Some(1), Some(4)),
            (Some(1), None),
            (None, Some(1)),
            (Some(2), Some(2)),
        ] {
            assert!(target_ratio_constraint(3, 1, invalid.0, invalid.1).is_err());
        }
    }

    #[test]
    fn target_ratio_residual_has_scale_independent_sign() {
        for ratio in [1.0_f64, 1.0e-2, 1.0e-8] {
            assert!(target_ratio_residual(ratio, ratio).unwrap().abs() < 1.0e-14);
            assert!(target_ratio_residual(ratio * 2.0, ratio).unwrap() > 0.0);
            assert!(target_ratio_residual(ratio * 0.5, ratio).unwrap() < 0.0);
        }
        assert!(target_ratio_residual(0.0, 1.0).is_err());
        assert!(target_ratio_residual(1.0, 0.0).is_err());
        assert!(target_ratio_residual(f64::NAN, 1.0).is_err());
    }

    #[test]
    fn initial_target_ratio_enforces_the_physical_amount_boundary() {
        assert_eq!(initial_target_ratio(2.0, 0.0).unwrap(), 0.0);
        assert_eq!(initial_target_ratio(2.0, 1.0).unwrap(), 0.5);
        for fixed in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(initial_target_ratio(fixed, 1.0).is_err());
        }
        for adjusting in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(initial_target_ratio(1.0, adjusting).is_err());
        }
    }

    #[test]
    fn target_ratio_solver_accepts_an_already_converged_initial_state() {
        let solution =
            solve_target_composition_ratio(0.25, test_constraint(), test_settings(4), |ratio| {
                Ok(trial_for_phase_ratio(ratio))
            })
            .unwrap();
        assert_eq!(solution.evaluations, 1);
        assert_eq!(solution.ratio, 0.25);
        assert!(solution.residual.abs() <= TARGET_RATIO_RESIDUAL_TOLERANCE);
    }

    #[test]
    fn target_ratio_solver_converges_for_an_ordinary_contraction() {
        let root = 0.2_f64.ln();
        let solution =
            solve_target_composition_ratio(2.0, test_constraint(), test_settings(12), |ratio| {
                let y = ratio.ln();
                Ok(trial_for_phase_ratio((root + 0.5 * (y - root)).exp()))
            })
            .unwrap();
        assert!((solution.ratio.ln() - root).abs() < 1.0e-6);
    }

    #[test]
    fn root_solver_converges_when_the_old_picard_iteration_diverges() {
        let root = 0.03_f64.ln();
        let mapping = |y: f64| root - 2.0 * (y - root);
        let mut old_picard = 0.8_f64.ln();
        for _ in 0..4 {
            old_picard = mapping(old_picard);
        }
        assert!((old_picard - root).abs() > 1.0);

        let solution =
            solve_target_composition_ratio(0.8, test_constraint(), test_settings(16), |ratio| {
                Ok(trial_for_phase_ratio(mapping(ratio.ln()).exp()))
            })
            .unwrap();
        assert!((solution.ratio.ln() - root).abs() < 1.0e-6);
        assert!(solution.used_secant);
    }

    #[test]
    fn target_ratio_solver_uses_secant_acceleration() {
        let root = 3.0_f64.ln();
        let solution =
            solve_target_composition_ratio(0.1, test_constraint(), test_settings(12), |ratio| {
                let y = ratio.ln();
                Ok(trial_for_phase_ratio((root + 0.8 * (y - root)).exp()))
            })
            .unwrap();
        assert!(solution.used_secant);
        assert!(solution.evaluations < 12);
    }

    #[test]
    fn target_ratio_solver_preserves_a_sign_bracket() {
        let solution =
            solve_target_composition_ratio(4.0, test_constraint(), test_settings(20), |ratio| {
                let y = ratio.ln();
                Ok(trial_for_phase_ratio((-0.5 * y).exp()))
            })
            .unwrap();
        assert!(solution.bracketed);
        assert!(solution.residual.abs() <= TARGET_RATIO_RESIDUAL_TOLERANCE);
    }

    #[test]
    fn target_ratio_solver_handles_trace_scale_ratios() {
        let root = 1.0e-8_f64.ln();
        let solution =
            solve_target_composition_ratio(1.0e-2, test_constraint(), test_settings(16), |ratio| {
                let y = ratio.ln();
                Ok(trial_for_phase_ratio((root + 0.25 * (y - root)).exp()))
            })
            .unwrap();
        assert!((solution.ratio.ln() - root).abs() <= TARGET_RATIO_RESIDUAL_TOLERANCE);
    }

    #[test]
    fn target_ratio_solver_handles_zero_boundaries_without_log_zero() {
        let exact =
            solve_target_composition_ratio(0.0, test_constraint(), test_settings(4), |_| {
                Ok(trial_for_phase_ratio(0.0))
            })
            .unwrap();
        assert_eq!(exact.ratio, 0.0);
        assert_eq!(exact.phase_ratio, 0.0);

        let seeded =
            solve_target_composition_ratio(0.0, test_constraint(), test_settings(4), |ratio| {
                Ok(trial_for_phase_ratio(if ratio == 0.0 {
                    0.25
                } else {
                    ratio
                }))
            })
            .unwrap();
        assert_eq!(seeded.ratio, 0.25);

        let boundary =
            solve_target_composition_ratio(0.5, test_constraint(), test_settings(4), |_| {
                Ok(trial_for_phase_ratio(0.0))
            })
            .unwrap();
        assert_eq!(boundary.ratio, 0.0);
    }

    #[test]
    fn target_ratio_solver_rejects_invalid_phase_fractions() {
        for trial in [
            TargetRatioTrial {
                fixed_fraction: 0.0,
                adjusting_fraction: 0.1,
            },
            TargetRatioTrial {
                fixed_fraction: f64::NAN,
                adjusting_fraction: 0.1,
            },
            TargetRatioTrial {
                fixed_fraction: 0.5,
                adjusting_fraction: f64::INFINITY,
            },
        ] {
            assert!(solve_target_composition_ratio(
                0.2,
                test_constraint(),
                test_settings(2),
                |_| Ok(trial),
            )
            .is_err());
        }
    }

    #[test]
    fn degenerate_secant_denominator_is_safely_bounded() {
        let error =
            solve_target_composition_ratio(1.0, test_constraint(), test_settings(6), |ratio| {
                Ok(trial_for_phase_ratio((ratio.ln() + 1.0).exp()))
            })
            .unwrap_err();
        assert!(error.description().contains("6 equilibrium evaluations"));
    }

    #[test]
    fn unbracketed_log_steps_respect_the_private_factor_limit() {
        let ratios = RefCell::new(Vec::new());
        let _ = solve_target_composition_ratio(1.0, test_constraint(), test_settings(2), |ratio| {
            ratios.borrow_mut().push(ratio);
            Ok(trial_for_phase_ratio(1.0e100))
        });
        let ratios = ratios.into_inner();
        assert_eq!(ratios.len(), 2);
        assert!(
            (ratios[1].ln() - ratios[0].ln()).abs() <= TARGET_RATIO_MAX_LOG_STEP + f64::EPSILON
        );
    }

    #[test]
    fn evaluation_budget_exhaustion_returns_contextual_error() {
        let error =
            solve_target_composition_ratio(1.0, test_constraint(), test_settings(3), |ratio| {
                Ok(trial_for_phase_ratio(ratio * 2.0))
            })
            .unwrap_err();
        let description = error.description();
        assert!(description.contains("did not converge"));
        assert!(description.contains("3 equilibrium evaluations"));
        assert!(description.contains("master phase=1"));
        assert!(description.contains("fixed component=1"));
        assert!(description.contains("adjusting component=2"));
    }

    #[test]
    fn nonconvergence_can_never_construct_a_success_value() {
        let result =
            solve_target_composition_ratio(1.0, test_constraint(), test_settings(1), |ratio| {
                Ok(trial_for_phase_ratio(ratio * 2.0))
            });
        assert!(matches!(result, Err(ChemAppError::OtherError(_))));
    }

    #[test]
    fn native_trial_errors_propagate_without_numerical_backtracking() {
        let result =
            solve_target_composition_ratio(1.0, test_constraint(), test_settings(8), |_| {
                Err(ChemAppError::NativeError(707))
            });
        assert!(matches!(result, Err(ChemAppError::NativeError(707))));
    }

    #[test]
    fn datafile_extension_is_case_insensitive_and_checked_before_native_calls() {
        assert_eq!(
            datafile_format_from_filename("system.DAT").unwrap(),
            DatafileFormat::Ascii
        );
        assert_eq!(
            datafile_format_from_filename("system.cSt").unwrap(),
            DatafileFormat::Transparent
        );
        assert_eq!(
            datafile_format_from_filename("system.bin").unwrap(),
            DatafileFormat::Binary
        );
        assert!(datafile_format_from_filename("system.unknown").is_err());
        assert!(datafile_format_from_filename("system").is_err());
    }

    #[test]
    fn constructors_report_missing_libraries_without_panicking() {
        let missing_library = "__chemapp_rs_missing_library_for_constructor_test__.dll";
        assert!(Calculator::from_library_unloaded(missing_library).is_err());
        assert!(Calculator::from_library(missing_library, "data/cosi.dat").is_err());
    }

    #[test]
    fn mapping_protocol_snapshots_each_successful_call_before_advancing() {
        use std::cell::RefCell;
        let events = RefCell::new(Vec::new());
        // Pop order is 1, 1, -1: the final negative terminal value must still
        // produce a snapshot before the loop stops.
        let continuations = RefCell::new(vec![-1i32, 1, 1]);
        let results = collect_mapping_results(
            "TF",
            "TN",
            |option| {
                events.borrow_mut().push(format!("call:{option}"));
                continuations
                    .borrow_mut()
                    .pop()
                    .ok_or_else(|| ChemAppError::OtherError("missing test continuation".to_owned()))
            },
            || {
                let ordinal = events
                    .borrow()
                    .iter()
                    .filter(|event| event.starts_with("call:"))
                    .count();
                events.borrow_mut().push(format!("snapshot:{ordinal}"));
                Ok(ordinal)
            },
        )
        .unwrap();

        assert_eq!(results, vec![1, 2, 3]);
        assert_eq!(
            events.into_inner(),
            vec![
                "call:TF",
                "snapshot:1",
                "call:TN",
                "snapshot:2",
                "call:TN",
                "snapshot:3",
            ]
        );
    }

    #[test]
    fn temperature_limit_retry_stops_after_a_successful_preferred_order() {
        let mut calls = Vec::new();
        set_temperature_limits_with_retry((100.0, 200.0), false, |option, _| {
            calls.push(option);
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, ["TLOW", "THIGH"]);
    }

    #[test]
    fn temperature_limit_retry_replays_both_reverse_bounds_after_first_failure() {
        let mut calls = Vec::new();
        let mut first = true;
        set_temperature_limits_with_retry((100.0, 200.0), false, |option, _| {
            calls.push(option);
            if first {
                first = false;
                Err(ChemAppError::NativeError(505))
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(calls, ["TLOW", "THIGH", "TLOW"]);
    }

    #[test]
    fn temperature_limit_retry_replays_both_reverse_bounds_after_second_failure() {
        let mut calls = Vec::new();
        set_temperature_limits_with_retry((100.0, 200.0), false, |option, _| {
            calls.push(option);
            if calls.len() == 2 {
                Err(ChemAppError::NativeError(505))
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(calls, ["TLOW", "THIGH", "THIGH", "TLOW"]);
    }

    #[test]
    fn temperature_limit_retry_preserves_both_bounded_failures() {
        let error = set_temperature_limits_with_retry((100.0, 200.0), false, |_, _| {
            Err(ChemAppError::NativeError(505))
        })
        .unwrap_err();
        match error {
            ChemAppError::RetryError {
                preferred,
                alternate,
                ..
            } => {
                assert!(matches!(*preferred, ChemAppError::NativeError(505)));
                assert!(matches!(*alternate, ChemAppError::NativeError(505)));
            }
            _ => panic!("expected bounded retry error"),
        }
    }

    #[test]
    fn stream_name_registry_has_one_owner_at_a_time() {
        let mut names = HashSet::new();
        assert!(names.insert("FEED".to_owned()));
        assert!(!names.insert("FEED".to_owned()));
        assert!(names.remove("FEED"));
        assert!(names.insert("FEED".to_owned()));
    }
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/
