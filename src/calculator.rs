// chemapp::calculator.rs

//! A high level submodule for easy operations on ChemApp library - avoid unnecessary boilerplate code. The user is still free to use both `native` and `Calculator` style function in a free manner.
//! An important feature of `Calculator` is the ability of predefining the composition basis - a useful feature, for example, in oxide systems, where system components are defined as elements, but the compositions should be entered as oxides (CaO, FeO, SiO2, etc).
use chemformula::Transform;
use nalgebra::{DVector, Dim, Storage, Vector};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::OsStr;
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

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

/// A higher-level abtraction entity.
#[derive(Debug)]
pub struct Calculator {
    /// a loaded instance of ChemApp engine
    pub engine: Engine,
    /// a copy of model parameters, allowing to restore delta inputs
    pub cache: Option<ParameterCache>,
    /// datafile
    pub file: String,
    /// a custom file to output errors
    pub nondefault_errunit: Option<(String, usize)>,
    /// isothermal calculation counter
    pub number_isothermal: usize,
    /// target calculation counter
    pub number_target_t: usize,
    /// instead of raw input using the system components basis, the user can define a custom formula basis; the transform is handled internally
    pub transform: Transform,
    /// The ERROR unit active before `redirect_error_to_temp` changed it.
    previous_errunit: Option<usize>,
    /// Names leased to live high-level `Stream` owners. ChemApp streams are
    /// name-addressed, so allowing duplicate owners would make `Drop` remove
    /// a stream still represented by another Rust value.
    pub(crate) active_stream_names: RefCell<HashSet<String>>,
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

impl Default for Calculator {
    fn default() -> Calculator {
        return Calculator {
            engine: Engine::default(),
            cache: None,
            file: r"".to_string(),
            nondefault_errunit: None,
            number_isothermal: 0,
            number_target_t: 0,
            transform: Transform::default(),
            previous_errunit: None,
            active_stream_names: RefCell::new(HashSet::new()),
        };
    }
}

/*******************************************************************************************************************************************************************************************************************************/
/*******************************************************************************************************************************************************************************************************************************/

impl Calculator {
    /// Read one typed interaction parameter from the current live TQGPAR matrix.
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
    /// results are stale until the caller explicitly calculates again.
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
        Transform::new(components, components, true).map_err(|error| {
            ChemAppError::OtherError(format!(
                "could not construct the ChemApp component identity transform: {error:?}"
            ))
        })
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Initialize a [`Calculator`] from a ChemApp dll file and a datafile
    pub fn from_library(libname: &str, datfile: &str) -> Result<Calculator, ChemAppError> {
        let engine = Engine::new(libname)?;
        Self::init_engine(&engine, datfile)?;
        let components = Self::component_names(&engine)?;
        let transform = Self::identity_transform(&components)?;
        return Ok(Calculator {
            engine: engine,
            cache: None,
            file: datfile.to_string(),
            nondefault_errunit: None,
            number_isothermal: 0,
            number_target_t: 0,
            transform,
            previous_errunit: None,
            active_stream_names: RefCell::new(HashSet::new()),
        });
    }

    /// Initializes a ChemApp interface without a datafile
    pub fn from_library_unloaded(libname: &str) -> Result<Calculator, ChemAppError> {
        let engine = Engine::new(libname)?;
        engine.tqini()?;
        return Ok(Calculator {
            engine: engine,
            cache: None,
            file: "".to_string(),
            nondefault_errunit: None,
            number_isothermal: 0,
            number_target_t: 0,
            transform: Transform::default(),
            previous_errunit: None,
            active_stream_names: RefCell::new(HashSet::new()),
        });
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

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Initializes the ChemApp interface and preconfigures it with the thermodynamic info from a datafile.
    pub fn init_engine(engine: &Engine, datfile: &str) -> Result<(), ChemAppError> {
        engine.tqini()?;
        Self::load_datafile(engine, datfile)?;
        return Ok(());
    }

    /// Loads one ChemApp thermochemical data-file through its format-specific
    /// native open/read/close sequence.
    ///
    /// The extension is validated before native state is queried. The configured
    /// `FILE` unit from `TQGIO` is then used consistently for opening and
    /// closing. Once an open succeeds, close is attempted even if the read
    /// fails; a dual failure retains the read error as the primary error.
    pub fn load_datafile(engine: &Engine, datfile: &str) -> Result<(), ChemAppError> {
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
    /// Set a formula transform for input compositions
    pub fn set_transform<T: AsRef<str>>(&mut self, basis: &[T]) -> Result<(), ChemAppError> {
        let components = Self::component_names(&self.engine)?;
        self.transform = Transform::new(&components, basis, true).map_err(|error| {
            ChemAppError::OtherError(format!(
                "could not construct the ChemApp composition transform: {error:?}"
            ))
        })?;
        return Ok(());
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
        return Ok(());
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Resets all input conditions to prepare for another calculation (with the same datafile).
    pub fn reset(&self) -> Result<(), ChemAppError> {
        self.engine.tqremc(-2)?;
        return Ok(());
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Global system properties accessor.
    pub fn system(&self) -> System<'_> {
        return System::new(self);
    }

    /// Iterates over system component indices.
    pub fn components(&self) -> Result<SystemComponentIterator<'_>, ChemAppError> {
        return SystemComponentIterator::new(self);
    }
    /// Iterates over phase indices.
    pub fn phases(&self) -> Result<PhaseIterator<'_>, ChemAppError> {
        return PhaseIterator::new(self);
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_report(self)
    }

    pub fn print_system(&self) -> Result<(), ChemAppError> {
        println!("{}", self.system().table_string()?);
        Ok(())
    }

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

    /// A simple isothermal calculation (temperature + initial composition in the pre-transformed basis).
    fn calculate_isothermal_(&self, x_i: &DVector<f64>, temp: f64) -> Result<(), ChemAppError> {
        self.reset()?;
        self.engine.tqsetc("T", 0, 0, temp)?;
        for k in 0..x_i.len() {
            self.engine.tqsetc("IA", 0, k + 1, x_i[k])?;
        }
        //self.engine.tqshow();
        self.engine.tqce(" ", 0, 0, (10.0, 6000.0))?;
        //self.number_isothermal += 1;
        return Ok(());
    }
    /// Perform a no-target isothermal calculation for an input composition and a temperature, use dynamic vectors; TODO check the composition transformations
    pub fn calculate_isothermal<D: Dim, S: Storage<f64, D>>(
        &self,
        compositions: &Vector<f64, D, S>,
        temp: f64,
    ) -> Result<(), ChemAppError> {
        return self.calculate_isothermal_(
            &self
                .transform
                .transform_final2init(compositions, false, false, false)
                .column(0)
                .into_owned(),
            temp,
        );
    }

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
        if fixed.is_some() != adjusting.is_some() {
            return Err(ChemAppError::OtherError(
                "both fixed and adjusting system components must be defined, or neither".to_owned(),
            ));
        }
        // set non-compositional conditions
        let nitermax = 10usize;
        let val = if precipitation { -0.5 } else { 0.0 };
        self.engine.tqsetc("A", target, 0, val)?;
        self.set_clim(interval, true)?;
        // set compositions
        let mut xvar: DVector<f64> = x_i.clone();
        //let mut xvarprev : DVector<f64> = xvar.clone();
        let mut xvarprev: DVector<f64>;
        match (fixed, adjusting) {
            (Some(sidxf), Some(sidxa)) => {
                for iter in 0..nitermax {
                    for k in 0..xvar.len() {
                        self.engine.tqsetc("IA", 0, k + 1, xvar[k])?;
                    }
                    //self.engine.tqshow()?;
                    self.engine.tqce("T", 0, 0, interval)?;
                    xvarprev = xvar.clone();
                    let xfold = xvar[sidxf - 1];
                    let xaold = xvar[sidxa - 1];
                    let xfnew = self.engine.tqgetr("XP", masterphase, sidxf)?;
                    let xanew = self.engine.tqgetr("XP", masterphase, sidxa)?;
                    let tliq = self.engine.tqgetr("T", 0, 0)?;
                    xvar[sidxa - 1] = xvar[sidxf - 1] * xanew / xfnew;
                    if iter > 0 {
                        xvar = (&xvar + &xvarprev) * 0.5;
                    }
                    //println!("iter = {:?}, tliq = {:?}, xfold = {:?}, xaold = {:?}, xfnew = {:?}, xanew = {:?}, xvarprev = {:?}, xvar = {:?}", &iter, &tliq, &xfold, &xaold, &xfnew, &xanew, &xvarprev, &xvar);
                    if (&xvar - &xvarprev).abs().sum() < 5e-3 {
                        return Ok(());
                    }
                }
            }
            (None, None) => {
                for k in 0..xvar.len() {
                    self.engine.tqsetc("IA", 0, k + 1, xvar[k])?;
                }
                // perform calculation
                //self.engine.tqshow()?;
                self.engine.tqce("T", 0, 0, interval)?;
            }
            _ => {
                return Err(ChemAppError::OtherError(
                    "both fixed and adjusting system components must be defined, or neither"
                        .to_owned(),
                ))
            }
        }
        //self.number_target_t += 1;
        return Ok(());
    }
    /// Perform a T-target calculation for an input composition and a temperature, use dynamic vectors; TODO check the composition transformations
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
        return self.calculate_target_t_(
            &self
                .transform
                .transform_final2init(compositions, false, false, false)
                .column(0)
                .into_owned(),
            masterphase,
            target,
            interval,
            precipitation,
            fixed,
            adjusting,
        );
    }

    fn calculate_target_x_from_left_(
        &self,
        x1: &DVector<f64>,
        x2: &DVector<f64>,
        temp: f64,
        target: usize,
    ) -> Result<(), ChemAppError> {
        todo!();
        /*
        let n_iter_max = 10;
        let mut x_initial = x1.clone();
        let mut x_other = x2;
        for k in 0..n_iter_max{
            x_initial = (&x_initial + x_other) * 0.5;
            self.calculate_isothermal(&x_initial, temp)?;
            if self.phases()?.phases_stable(&self).any(|pid| pid == target) {
                if self.phases()?.phases_stable(&self).count() > 1 {
                    return Ok(());
                }
                //x_other = x2;
            }
            x_other = x1;
        }
        return Err(ChemAppError::OtherError("Cannot converge X target".to_string()));
        */
    }
    /// Perform a composition search starting from `x1` until a required phase is met, use dynamic vectors; TODO check the composition transformations
    pub fn calculate_target_x_from_left<D: Dim, S: Storage<f64, D>>(
        &self,
        x1: &Vector<f64, D, S>,
        x2: &Vector<f64, D, S>,
        temp: f64,
        target: usize,
    ) -> Result<(), ChemAppError> {
        return self.calculate_target_x_from_left_(
            &self
                .transform
                .transform_final2init(x1, false, false, false)
                .column(0)
                .into_owned(),
            &self
                .transform
                .transform_final2init(x2, false, false, false)
                .column(0)
                .into_owned(),
            temp,
            target,
        );
    }

    /***************************************************************************************************************************************************************************************************************************/
    /***************************************************************************************************************************************************************************************************************************/

    /// Create a snapshot of the current state.
    pub fn snapshot(&self) -> Result<CalculatorSnapshot, ChemAppError> {
        CalculatorSnapshot::new(self)
    }

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

    pub fn mapping_temperature(
        &self,
        tmin: f64,
        tmax: f64,
        list: bool,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping_temperature_with_options(tmin, tmax, list, SnapshotOptions::all())
    }

    pub fn mapping_temperature_with_options(
        &self,
        tmin: f64,
        tmax: f64,
        list: bool,
        options: SnapshotOptions,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping("TF", "TN", 0, 0, (tmin, tmax), list, options)
    }

    pub fn mapping_pressure(
        &self,
        pmin: f64,
        pmax: f64,
        list: bool,
    ) -> Result<Vec<CalculatorSnapshot>, ChemAppError> {
        self.mapping_pressure_with_options(pmin, pmax, list, SnapshotOptions::all())
    }

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
        match &self.nondefault_errunit {
            Some((filename, unit)) => {
                if let Some(previous_unit) = self.previous_errunit {
                    let _ = self.engine.tqcio("ERROR", previous_unit);
                }
                let _ = self.engine.tqclos(*unit);
                let _ = std::fs::remove_file(&filename);
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
