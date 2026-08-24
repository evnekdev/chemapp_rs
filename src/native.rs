// chemapp::native.rs

//! This submodule exports ChemApp functions as-is with the minimal changes in the function signatures to adapt to the Rust infrastructure.
#![allow(unused_imports)]

extern crate libloading;

use function_name::named;
use libloading::{Library, Symbol};
use std::cell::Cell;
use std::cmp::min;
use std::ffi::CString;
use std::str::from_utf8;

use crate::abi::{
    chemapp_int_array_to_i32, chemapp_int_to_i32, chemapp_int_to_u32, chemapp_int_to_usize,
    cstring_character_input, i32_to_chemapp_int, usize_to_chemapp_character_length,
    usize_to_chemapp_int, wrap_nonnegative_result, wrap_result, ChemAppInt, ChemAppLen,
};
use crate::defs::{FUNCSUNIX32, FUNCSUNIX64, FUNCSWIN32, FUNCSWIN64};
use crate::error::ChemAppError;
use crate::{SystemDimensions, TransparentHeader};

const NAME_LENGTH_MAX: usize = 25;
const TQGTID_CHARACTER_LENGTH: usize = 255;
const TQGTNM_CHARACTER_LENGTH: usize = 80;
const TQGTRH_PROGRAM_NAME_LENGTH: usize = 40;
const TQGTRH_USER_ID_LENGTH: usize = 255;
const TQGTRH_TEXT_LENGTH: usize = 80;
const TQERR_RECORD_LENGTH: usize = 80;
const TQERR_RECORD_COUNT: usize = 3;

// Rust buffer capacity is intentionally kept as `usize` above. These values
// are the corresponding raw Fortran declarations and must cross FFI as the
// target-specific `ChemAppLen` type.
const NAME_NATIVE_CHARACTER_LENGTH: ChemAppLen = 25;
const TQGTID_NATIVE_CHARACTER_LENGTH: ChemAppLen = 255;
const TQGTNM_NATIVE_CHARACTER_LENGTH: ChemAppLen = 80;
const TQGTRH_PROGRAM_NAME_NATIVE_LENGTH: ChemAppLen = 40;
const TQGTRH_USER_ID_NATIVE_LENGTH: ChemAppLen = 255;
const TQGTRH_TEXT_NATIVE_LENGTH: ChemAppLen = 80;
const TQERR_NATIVE_RECORD_LENGTH: ChemAppLen = 80;
const TQLPAR_NATIVE_RECORD_LENGTH: ChemAppLen = 156;
// TQLPAR record capacity and TQGPAR's Fortran leading dimension are queried
// from TQSIZE. This avoids baking the checked 7.14 build's ND/NE/NI values
// into the caller-side buffers.
const TQGPAR_VALUE_CAPACITY: usize = 28;

/*********************************************************************************************************************************************************************************************************/
/*********************************************************************************************************************************************************************************************************/

fn func_alias(name: &'static str) -> &'static str {
    #[cfg(all(target_family = "windows", target_pointer_width = "32"))]
    let funcs = &FUNCSWIN32;

    #[cfg(all(target_family = "windows", target_pointer_width = "64"))]
    let funcs = &FUNCSWIN64;

    #[cfg(all(target_family = "unix", target_pointer_width = "32"))]
    let funcs = &FUNCSUNIX32;

    #[cfg(all(target_family = "unix", target_pointer_width = "64"))]
    let funcs = &FUNCSUNIX64;

    funcs[name]
}

/// Converts exactly one fixed-width Fortran CHARACTER result to a Rust string.
///
/// `character_length` is the Fortran declaration, not necessarily the Rust
/// allocation length. ChemApp may blank-pad the record and is not required to
/// NUL-terminate it, so this deliberately never examines a convenience byte
/// beyond the declared record. Internal spaces are data: only trailing
/// Fortran padding is removed. Fixed-width output must never be parsed by
/// looking for its first space.
fn fixed_fortran_string(buffer: &[u8], character_length: usize) -> Result<String, ChemAppError> {
    if buffer.len() < character_length {
        return Err(ChemAppError::OtherError(format!(
            "Fortran CHARACTER buffer has length {}, but {} bytes are required",
            buffer.len(),
            character_length
        )));
    }

    let record = &buffer[..character_length];
    let nul_end = record
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(record.len());
    let end = record[..nul_end]
        .iter()
        .rposition(|byte| *byte != b' ')
        .map(|last| last + 1)
        .unwrap_or(0);
    return Ok(from_utf8(&record[..end])?.to_owned());
}

/// Converts TQERR's three 80-character Fortran records without treating the
/// contiguous 240-byte Rust allocation as a single CHARACTER*240 value.
fn tqerr_message(buffer: &[u8]) -> Result<String, ChemAppError> {
    if buffer.len() < TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT {
        return Err(ChemAppError::OtherError(
            "TQERR buffer is too short".to_owned(),
        ));
    }

    let mut records = Vec::new();
    for record in
        buffer[..TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT].chunks_exact(TQERR_RECORD_LENGTH)
    {
        let text = fixed_fortran_string(record, TQERR_RECORD_LENGTH)?;
        if !text.is_empty() {
            records.push(text);
        }
    }
    return Ok(records.join("\n"));
}

/// Reconstructs logical TQGPAR expression rows from Fortran column-major
/// storage. TQSIZE supplies the compiled leading dimension even when returned
/// `NOEXPR` is smaller.
fn tqgpar_values(
    raw: &[f64],
    leading_dimension: usize,
    noexpr: usize,
    nvala: usize,
) -> Result<Vec<Vec<f64>>, ChemAppError> {
    if noexpr > leading_dimension || nvala > TQGPAR_VALUE_CAPACITY {
        return Err(ChemAppError::OtherError(format!(
			"TQGPAR returned dimensions {noexpr}x{nvala} for a {leading_dimension}x{TQGPAR_VALUE_CAPACITY} buffer"
		)));
    }
    let required = leading_dimension
        .checked_mul(nvala)
        .ok_or_else(|| ChemAppError::OtherError("TQGPAR raw extent overflowed usize".to_owned()))?;
    if raw.len() < required {
        return Err(ChemAppError::OtherError(
            "TQGPAR raw buffer is shorter than its declared Fortran extent".to_owned(),
        ));
    }
    Ok((0..noexpr)
        .map(|expression| {
            (0..nvala)
                .map(|value| raw[expression + value * leading_dimension])
                .collect()
        })
        .collect())
}

macro_rules! raw_chemapp_ints {
	($($value:ident),+ $(,)?) => {
		$(let $value = usize_to_chemapp_int($value)?;)+
	};
}

/// Raw `LI` storage for `TQSIZE` and `TQUSED`.
struct RawSystemDimensions {
    nconstituents: ChemAppInt,
    ncomponents: ChemAppInt,
    nmixtures: ChemAppInt,
    nexcess_gibbs: ChemAppInt,
    nexcess_magnetic: ChemAppInt,
    nsublattices: ChemAppInt,
    nspecies: ChemAppInt,
    nconstituents_mqm: ChemAppInt,
    nranges_constituent: ChemAppInt,
    nranges: ChemAppInt,
    ndependent: ChemAppInt,
}

impl RawSystemDimensions {
    fn new() -> Self {
        Self {
            nconstituents: 0,
            ncomponents: 0,
            nmixtures: 0,
            nexcess_gibbs: 0,
            nexcess_magnetic: 0,
            nsublattices: 0,
            nspecies: 0,
            nconstituents_mqm: 0,
            nranges_constituent: 0,
            nranges: 0,
            ndependent: 0,
        }
    }
}

/// Adapts `TQSIZE`/`TQUSED` raw `LI` outputs only after the native call has
/// succeeded. `SystemDimensions` intentionally keeps its established public
/// `i32` fields, so it must never be handed to a source-modelled `long*` ABI
/// directly on a fallback target.
fn system_dimensions_from_raw(
    values: RawSystemDimensions,
) -> Result<SystemDimensions, ChemAppError> {
    Ok(SystemDimensions {
        constituents: chemapp_int_to_i32(values.nconstituents)?,
        system_components: chemapp_int_to_i32(values.ncomponents)?,
        mixture_phases: chemapp_int_to_i32(values.nmixtures)?,
        excess_gibbs_coefficients_per_phase: chemapp_int_to_i32(values.nexcess_gibbs)?,
        excess_magnetic_coefficients_per_phase: chemapp_int_to_i32(values.nexcess_magnetic)?,
        sublattices_per_phase: chemapp_int_to_i32(values.nsublattices)?,
        constituents_per_sublattice: chemapp_int_to_i32(values.nspecies)?,
        gkf_mqm_oxide_constituents_per_phase: chemapp_int_to_i32(values.nconstituents_mqm)?,
        equations_per_constituent: chemapp_int_to_i32(values.nranges_constituent)?,
        equations: chemapp_int_to_i32(values.nranges)?,
        pt_dependent_volume_constituents: chemapp_int_to_i32(values.ndependent)?,
    })
}

/*********************************************************************************************************************************************************************************************************/
/*********************************************************************************************************************************************************************************************************/

/// One dynamically loaded, stateful ChemApp native-library instance.
///
/// The `tq...` methods adapt ChemApp's raw Fortran ABI without changing its
/// one-based indices, call ordering, units, or native state semantics. Loading
/// is fallible, so `Engine` deliberately has no `Default` implementation; use
/// [`Engine::new`] with an explicit compatible library path.
///
/// ChemApp mutates native state even through methods taking `&self`. `Engine`
/// is therefore deliberately not [`Sync`]: one instance must never be called
/// concurrently. Ownership may be moved to another thread, but parallel work
/// requires independent supported ChemApp library instances/copies.
///
/// ```compile_fail
/// use chemapp_rs::Engine;
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<Engine>();
/// ```
#[derive(Debug)]
pub struct Engine {
    pub(crate) library_name: String,
    library: Library,
    // Cell is zero-sized and Send but !Sync. It expresses ChemApp's mutable,
    // non-reentrant per-library state without adding a locking fiction.
    _not_sync: Cell<()>,
}

impl Engine {
    /*****************************************************************************************************************************************************************************************************/
    /// Initializes a new instance of `Engine` from a DLL path or name. In case a name only is used, the DLL has to be discoverable in PATH system variable (modify the system environment variables if it is not the case).
    pub fn new(library_name: &str) -> Result<Engine, ChemAppError> {
        return Ok(Engine {
            library_name: String::from(library_name),
            library: unsafe { Library::new(library_name)? },
            _not_sync: Cell::new(()),
        });
    }

    /*****************************************************************************************************************************************************************************************************/
    /// INITIALIZE-INTERFACE
    #[named]
    pub fn tqini(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-VERSION-NUMBER
    #[named]
    pub fn tqvers(&self) -> Result<i32, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut vers: ChemAppInt = 0;
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(vers: &mut ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut vers, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(vers: &mut ChemAppInt, errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut vers, &mut errcode);
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return chemapp_int_to_i32(vers);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-COPYRIGHT-MESSAGE
    #[named]
    pub fn tqcprt(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHECK-IF-CHEMAPP-LIGHT
    #[named]
    pub fn tqlite(&self) -> Result<bool, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut lite: ChemAppInt = 0;
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(lite: &mut ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut lite, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(lite: &mut ChemAppInt, errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut lite, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result(lite > 0, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-USER-ID
    #[named]
    pub fn tqgtid(&self) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        // The GTT bridge declares CHARACTER*255. No spare Rust byte is passed
        // to Fortran as part of that declaration.
        let mut cstring: [u8; TQGTID_CHARACTER_LENGTH] = [0; TQGTID_CHARACTER_LENGTH];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cstring: &mut u8,
                    length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                TQGTID_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(cstring: &mut u8, errcode: &mut ChemAppInt, length: ChemAppLen) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                &mut errcode,
                TQGTID_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(
            fixed_fortran_string(&cstring, TQGTID_CHARACTER_LENGTH)?,
            errcode,
        );
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-USER-NAME
    #[named]
    pub fn tqgtnm(&self) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        let mut cstring: [u8; TQGTNM_CHARACTER_LENGTH] = [0; TQGTNM_CHARACTER_LENGTH];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cstring: &mut u8,
                    length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                TQGTNM_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(cstring: &mut u8, errcode: &mut ChemAppInt, length: ChemAppLen) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                &mut errcode,
                TQGTNM_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(
            fixed_fortran_string(&cstring, TQGTNM_CHARACTER_LENGTH)?,
            errcode,
        );
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-PROGRAM-ID
    #[named]
    pub fn tqgtpi(&self) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        let mut cstring: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cstring: &mut u8,
                    length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut cstring[0], NAME_NATIVE_CHARACTER_LENGTH, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(cstring: &mut u8, errcode: &mut ChemAppInt, length: ChemAppLen) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut cstring[0], &mut errcode, NAME_NATIVE_CHARACTER_LENGTH);
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cstring, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-HASP-DONGLE-INFO
    #[named]
    pub fn tqgthi(&self) -> Result<(String, i32), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode: ChemAppInt = 0;
        let mut hid: ChemAppInt = 0;
        let mut cstring: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cstring: &mut u8,
                    length: ChemAppLen,
                    hid: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut hid,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cstring: &mut u8,
                    hid: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cstring[0],
                &mut hid,
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return Ok((
            fixed_fortran_string(&cstring, NAME_LENGTH_MAX)?,
            chemapp_int_to_i32(hid)?,
        ));
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-EXPIRATION-MONTH-AND-YEAR
    #[named]
    pub fn tqgted(&self) -> Result<(u32, u32), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut month: ChemAppInt = 0;
        let mut year: ChemAppInt = 0;
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    month: &mut ChemAppInt,
                    year: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut month, &mut year, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    month: &mut ChemAppInt,
                    year: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut month, &mut year, &mut errcode);
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return Ok((chemapp_int_to_u32(month)?, chemapp_int_to_u32(year)?));
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SET-CONFIGURATION-OPTION
    #[named]
    pub fn tqconf(
        &self,
        option: &str,
        valuea: usize,
        valueb: usize,
        valuec: usize,
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(valuea, valueb, valuec);
        let coption: CString = CString::new(option)?;
        let coption_length = cstring_character_input(&coption)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    coption: *const u8,
                    coption_length: ChemAppLen,
                    valuea: &ChemAppInt,
                    valueb: &ChemAppInt,
                    valuec: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                coption_length,
                &valuea,
                &valueb,
                &valuec,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    coption: *const u8,
                    valuea: &ChemAppInt,
                    valueb: &ChemAppInt,
                    valuec: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    coption_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &valuea,
                &valueb,
                &valuec,
                &mut errcode,
                coption_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-ARRAY-SIZES
    #[named]
    pub fn tqsize(&self) -> Result<SystemDimensions, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut dims = RawSystemDimensions::new();
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    na: &mut ChemAppInt,
                    nb: &mut ChemAppInt,
                    nc: &mut ChemAppInt,
                    nd: &mut ChemAppInt,
                    ne: &mut ChemAppInt,
                    nf: &mut ChemAppInt,
                    ng: &mut ChemAppInt,
                    nh: &mut ChemAppInt,
                    ni: &mut ChemAppInt,
                    nj: &mut ChemAppInt,
                    nk: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut dims.nconstituents,
                &mut dims.ncomponents,
                &mut dims.nmixtures,
                &mut dims.nexcess_gibbs,
                &mut dims.nexcess_magnetic,
                &mut dims.nsublattices,
                &mut dims.nspecies,
                &mut dims.nconstituents_mqm,
                &mut dims.nranges_constituent,
                &mut dims.nranges,
                &mut dims.ndependent,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    na: &mut ChemAppInt,
                    nb: &mut ChemAppInt,
                    nc: &mut ChemAppInt,
                    nd: &mut ChemAppInt,
                    ne: &mut ChemAppInt,
                    nf: &mut ChemAppInt,
                    ng: &mut ChemAppInt,
                    nh: &mut ChemAppInt,
                    ni: &mut ChemAppInt,
                    nj: &mut ChemAppInt,
                    nk: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut dims.nconstituents,
                &mut dims.ncomponents,
                &mut dims.nmixtures,
                &mut dims.nexcess_gibbs,
                &mut dims.nexcess_magnetic,
                &mut dims.nsublattices,
                &mut dims.nspecies,
                &mut dims.nconstituents_mqm,
                &mut dims.nranges_constituent,
                &mut dims.nranges,
                &mut dims.ndependent,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return system_dimensions_from_raw(dims);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-CURRENT-DIMENSIONS
    #[named]
    pub fn tqused(&self) -> Result<SystemDimensions, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut dims = RawSystemDimensions::new();
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    na: &mut ChemAppInt,
                    nb: &mut ChemAppInt,
                    nc: &mut ChemAppInt,
                    nd: &mut ChemAppInt,
                    ne: &mut ChemAppInt,
                    nf: &mut ChemAppInt,
                    ng: &mut ChemAppInt,
                    nh: &mut ChemAppInt,
                    ni: &mut ChemAppInt,
                    nj: &mut ChemAppInt,
                    nk: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut dims.nconstituents,
                &mut dims.ncomponents,
                &mut dims.nmixtures,
                &mut dims.nexcess_gibbs,
                &mut dims.nexcess_magnetic,
                &mut dims.nsublattices,
                &mut dims.nspecies,
                &mut dims.nconstituents_mqm,
                &mut dims.nranges_constituent,
                &mut dims.nranges,
                &mut dims.ndependent,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    na: &mut ChemAppInt,
                    nb: &mut ChemAppInt,
                    nc: &mut ChemAppInt,
                    nd: &mut ChemAppInt,
                    ne: &mut ChemAppInt,
                    nf: &mut ChemAppInt,
                    ng: &mut ChemAppInt,
                    nh: &mut ChemAppInt,
                    ni: &mut ChemAppInt,
                    nj: &mut ChemAppInt,
                    nk: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut dims.nconstituents,
                &mut dims.ncomponents,
                &mut dims.nmixtures,
                &mut dims.nexcess_gibbs,
                &mut dims.nexcess_magnetic,
                &mut dims.nsublattices,
                &mut dims.nspecies,
                &mut dims.nconstituents_mqm,
                &mut dims.nranges_constituent,
                &mut dims.nranges,
                &mut dims.ndependent,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return system_dimensions_from_raw(dims);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-VALUE-OF-INPUT-OUTPUT-OPTION
    #[named]
    pub fn tqgio(&self, option: &str) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        let mut num = 0;
        let coption: CString = CString::new(option)?;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    num: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &mut num,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    num: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &mut num,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(num, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-INPUT-OPTION
    #[named]
    pub fn tqcio(&self, option: &str, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let coption: CString = CString::new(option)?;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &unit,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &unit,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// READ-DATA-FILE
    #[named]
    pub fn tqrfil(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// READ-BINARY-DATA-FILE
    #[named]
    pub fn tqrbin(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// READ-TRANSPARENT-DATA-FILE
    #[named]
    pub fn tqrcst(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// OPEN-FILE
    #[named]
    pub fn tqopen(&self, filename: &str, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let cfilename: CString = CString::new(filename)?;
        let cfilename_length = cstring_character_input(&cfilename)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cfilename: *const u8,
                    filename_length: ChemAppLen,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cfilename)?.0,
                cfilename_length,
                &unit,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cfilename: *const u8,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    filename_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cfilename)?.0,
                &unit,
                &mut errcode,
                cfilename_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// WRITE-STRING
    #[named]
    pub fn tqwstr(&self, option: &str, text: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let coption: CString = CString::new(option)?;
        let ctext: CString = CString::new(text)?;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    text: *const u8,
                    text_len: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                cstring_character_input(&ctext)?.0,
                cstring_character_input(&ctext)?.1,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    text: *const u8,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                    text_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&ctext)?.0,
                &mut errcode,
                cstring_character_input(&coption)?.1,
                cstring_character_input(&ctext)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// OPEN-ASCII-DATA-FILE
    #[named]
    pub fn tqopna(&self, name: &str, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let cname: CString = CString::new(name)?;
        let cname_length = cstring_character_input(&cname)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cname: *const u8,
                    cfilename_length: ChemAppLen,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cname_length,
                &unit,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cname: *const u8,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    cfilename_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &unit,
                &mut errcode,
                cname_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// OPEN-BINARY-DATA-FILE
    #[named]
    pub fn tqopnb(&self, name: &str, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let cname: CString = CString::new(name)?;
        let cname_length = cstring_character_input(&cname)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cname: *const u8,
                    cname_length: ChemAppLen,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cname_length,
                &unit,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cname: *const u8,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &unit,
                &mut errcode,
                cname_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// OPEN-TRANSPARENT-DATA-FILE
    #[named]
    pub fn tqopnt(&self, name: &str, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let cname: CString = CString::new(name)?;
        let cname_length = cstring_character_input(&cname)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cname: *const u8,
                    cname_length: ChemAppLen,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cname_length,
                &unit,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cname: *const u8,
                    unit: &ChemAppInt,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &unit,
                &mut errcode,
                cname_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CLOSE-FILE
    #[named]
    pub fn tqclos(&self, unit: usize) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(unit);
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(unit: &ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&unit, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(unit: &ChemAppInt, errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&unit, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-TRANSPARENT-FILE-HEADER-INFO
    #[named]
    pub fn tqgtrh(&self) -> Result<TransparentHeader, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut cver: ChemAppInt = 0;
        let mut cnwp: [u8; TQGTRH_PROGRAM_NAME_LENGTH] = [0; TQGTRH_PROGRAM_NAME_LENGTH];
        let mut cvnw: [ChemAppInt; 3] = [0; 3];
        let mut cnrp: [u8; TQGTRH_PROGRAM_NAME_LENGTH] = [0; TQGTRH_PROGRAM_NAME_LENGTH];
        let mut cvnr: [ChemAppInt; 3] = [0; 3];
        let mut cdtc: [ChemAppInt; 6] = [0; 6];
        let mut cdte: [ChemAppInt; 6] = [0; 6];
        let mut cid: [u8; TQGTRH_USER_ID_LENGTH] = [0; TQGTRH_USER_ID_LENGTH];
        let mut cusr: [u8; TQGTRH_TEXT_LENGTH] = [0; TQGTRH_TEXT_LENGTH];
        let mut crem: [u8; TQGTRH_TEXT_LENGTH] = [0; TQGTRH_TEXT_LENGTH];
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    &mut ChemAppInt,
                    &mut u8,
                    ChemAppLen,
                    &mut ChemAppInt,
                    &mut u8,
                    ChemAppLen,
                    &mut ChemAppInt,
                    &mut ChemAppInt,
                    &mut ChemAppInt,
                    &mut u8,
                    ChemAppLen,
                    &mut u8,
                    ChemAppLen,
                    &mut u8,
                    ChemAppLen,
                    &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cver,
                &mut cnwp[0],
                TQGTRH_PROGRAM_NAME_NATIVE_LENGTH,
                &mut cvnw[0],
                &mut cnrp[0],
                TQGTRH_PROGRAM_NAME_NATIVE_LENGTH,
                &mut cvnr[0],
                &mut cdtc[0],
                &mut cdte[0],
                &mut cid[0],
                TQGTRH_USER_ID_NATIVE_LENGTH,
                &mut cusr[0],
                TQGTRH_TEXT_NATIVE_LENGTH,
                &mut crem[0],
                TQGTRH_TEXT_NATIVE_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    &mut ChemAppInt,
                    &mut u8,
                    &mut ChemAppInt,
                    &mut u8,
                    &mut ChemAppInt,
                    &mut ChemAppInt,
                    &mut ChemAppInt,
                    &mut u8,
                    &mut u8,
                    &mut u8,
                    &mut ChemAppInt,
                    ChemAppLen,
                    ChemAppLen,
                    ChemAppLen,
                    ChemAppLen,
                    ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &mut cver,
                &mut cnwp[0],
                &mut cvnw[0],
                &mut cnrp[0],
                &mut cvnr[0],
                &mut cdtc[0],
                &mut cdte[0],
                &mut cid[0],
                &mut cusr[0],
                &mut crem[0],
                &mut errcode,
                TQGTRH_PROGRAM_NAME_NATIVE_LENGTH,
                TQGTRH_PROGRAM_NAME_NATIVE_LENGTH,
                TQGTRH_USER_ID_NATIVE_LENGTH,
                TQGTRH_TEXT_NATIVE_LENGTH,
                TQGTRH_TEXT_NATIVE_LENGTH,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        let header: TransparentHeader = TransparentHeader {
            version: chemapp_int_to_i32(cver)?,
            name_writing_program: fixed_fortran_string(&cnwp, TQGTRH_PROGRAM_NAME_LENGTH)?,
            version_writing_program: chemapp_int_array_to_i32(cvnw)?,
            name_reading_program: fixed_fortran_string(&cnrp, TQGTRH_PROGRAM_NAME_LENGTH)?,
            minversion_reading_program: chemapp_int_array_to_i32(cvnr)?,
            creation_date: chemapp_int_array_to_i32(cdtc)?,
            expiry_date: chemapp_int_array_to_i32(cdte)?,
            user_ids_allowed: fixed_fortran_string(&cid, TQGTRH_USER_ID_LENGTH)?,
            license_holders_allowed: fixed_fortran_string(&cusr, TQGTRH_TEXT_LENGTH)?,
            remark: fixed_fortran_string(&crem, TQGTRH_TEXT_LENGTH)?,
        };
        return Ok(header);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-SYSTEM-UNIT
    #[named]
    pub fn tqgsu(&self, option: &str) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        let coption: CString = CString::new(option)?;
        // CString::as_bytes excludes the terminating NUL and therefore matches
        // the C bridge's strlen(OPTION), including for an empty option.
        let option_length = cstring_character_input(&coption)?.1;
        let mut cunit: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    unit: &mut u8,
                    unit_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &mut cunit[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    unit: &mut u8,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                    unit_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &mut cunit[0],
                &mut errcode,
                option_length,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cunit, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-SYSTEM-UNIT
    #[named]
    pub fn tqcsu(&self, option: &str, unit: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let coption: CString = CString::new(option)?;
        let cunit: CString = CString::new(unit)?;
        let option_length = cstring_character_input(&coption)?.1;
        let unit_length = cstring_character_input(&cunit)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    unit: *const u8,
                    unit_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                cstring_character_input(&cunit)?.0,
                unit_length,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    unit: *const u8,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                    unit_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&cunit)?.0,
                &mut errcode,
                option_length,
                unit_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-INDEX-NUMBER-OF-SYSTEM-COMPONENT
    #[named]
    pub fn tqinsc(&self, name: &str) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        let cname: CString = CString::new(name)?;
        let name_length = cstring_character_input(&cname)?.1;
        let mut indexs = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    name: *const u8,
                    name_length: ChemAppLen,
                    indexs: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                name_length,
                &mut indexs,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    name: *const u8,
                    indexs: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    name_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &mut indexs,
                &mut errcode,
                name_length,
            );
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(indexs, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NAME-OF-SYSTEM-COMPONENT
    #[named]
    pub fn tqgnsc(&self, indexs: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexs);
        let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexs: &ChemAppInt,
                    name: &mut u8,
                    name_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexs,
                &mut cname[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexs: &ChemAppInt,
                    name: &mut u8,
                    errcode: &mut ChemAppInt,
                    name_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexs,
                &mut cname[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-NAME-OF-SYSTEM-COMPONENT
    #[named]
    pub fn tqcnsc(&self, indexs: usize, name: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexs);
        let cname: CString = CString::new(name)?;
        let name_length = cstring_character_input(&cname)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexs: &ChemAppInt,
                    name: *const u8,
                    name_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexs,
                cstring_character_input(&cname)?.0,
                name_length,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexs: &ChemAppInt,
                    name: *const u8,
                    errcode: &mut ChemAppInt,
                    name_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexs,
                cstring_character_input(&cname)?.0,
                &mut errcode,
                name_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NUMBER-OF-SYSTEM-COMPONENTS
    #[named]
    pub fn tqnosc(&self) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut nscom = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(nscom: &mut ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut nscom, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(nscom: &mut ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut nscom, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(nscom, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-STOICHIOMETRY-OF-SYSTEM-COMPONENT
    #[named]
    pub fn tqstsc(&self, indexs: usize) -> Result<(Vec<f64>, f64), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexs);
        let ncomp = self.tqnosc()?;
        let mut stoi: Vec<f64> = vec![0.0; ncomp];
        let mut wmass = 0.0f64;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexs: &ChemAppInt,
                    stoi: &mut f64,
                    wmass: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexs, &mut stoi[0], &mut wmass, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexs: &ChemAppInt,
                    stoi: &mut f64,
                    wmass: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexs, &mut stoi[0], &mut wmass, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((stoi, wmass), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-SYSTEM-COMPONENTS
    #[named]
    pub fn tqcsc(&self, names: &[&str]) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let nsyscom = self.tqnosc()?;
        let length: usize = 24;
        let character_length = usize_to_chemapp_character_length(length)?;
        let mut namememory: Vec<u8> = vec![32; (nsyscom + 1) * length];
        let mut cname: CString;
        let mut cbytes: &[u8];
        let mut size: usize;
        for k in 0..names.len() {
            cname = CString::new(names[k])?;
            cbytes = cname.as_bytes();
            size = min(length, cbytes.len());
            namememory[k * length..k * length + size].clone_from_slice(&cbytes[0..size]);
            //namememory[(k+1)*length] = 0;
        }
        for k in names.len()..nsyscom {
            cname = CString::new(self.tqgnsc(k + 1)?)?;
            cbytes = cname.as_bytes();
            size = min(length, cbytes.len());
            namememory[k * length..k * length + size].clone_from_slice(&cbytes[0..size]);
            //namememory[(k+1)*length] = 0;
        }
        //println!("namememory = {:?}", namememory);
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    names: *const u8,
                    names_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(namememory.as_ptr(), character_length, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    names: *const u8,
                    errcode: &mut ChemAppInt,
                    names_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(namememory.as_ptr(), &mut errcode, character_length);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-INDEX-NUMBER-OF-PHASE
    #[named]
    pub fn tqinp(&self, name: &str) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        let cname: CString = CString::new(name)?;
        let cname_length = cstring_character_input(&cname)?.1;
        let mut indexp = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cname: *const u8,
                    cname_length: ChemAppLen,
                    indexp: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cname_length,
                &mut indexp,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cname: *const u8,
                    indexp: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &mut indexp,
                &mut errcode,
                cname_length,
            );
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(indexp, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NAME-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqgnp(&self, indexp: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    cname: &mut u8,
                    cname_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cname[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    cname: &mut u8,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cname[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-MODEL-NAME-OF-PHASE
    #[named]
    pub fn tqmodl(&self, indexp: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    cname: &mut u8,
                    cname_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cname[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    cname: &mut u8,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cname[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NUMBER-OF-PHASES
    #[named]
    pub fn tqnop(&self) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        let mut nphase = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(nphase: &mut ChemAppInt, errcode: &mut ChemAppInt),
            > = self.library.get(fname.as_bytes())?;
            func(&mut nphase, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(nphase: &mut ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut nphase, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(nphase, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-INDEX-NUMBER-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqinpc(&self, indexp: usize, name: &str) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut indexc = 0;
        let mut errcode = 0;
        let cname: CString = CString::new(name)?;
        let cname_length = cstring_character_input(&cname)?.1;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cname: *const u8,
                    name_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cname_length,
                &indexp,
                &mut indexc,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cname: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    name_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &indexp,
                &mut indexc,
                &mut errcode,
                cname_length,
            );
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(indexc, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NAME-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqgnpc(&self, indexp: usize, indexc: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cname: &mut u8,
                    cname_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                &mut cname[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cname: &mut u8,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                &mut cname[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// PHASE-CONSTITUENT-IS-INCOMING-SPECIES
    #[named]
    pub fn tqpcis(&self, indexp: usize, indexc: usize) -> Result<bool, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let mut value = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    value: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut value, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    value: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut value, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result(value > 0, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NUMBER-OF-PHASE-CONSTITUENTS
    #[named]
    pub fn tqnopc(&self, indexp: usize) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut nconst = 0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    nconst: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &mut nconst, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    nconst: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &mut nconst, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(nconst, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-STOICHIOMETRY-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqstpc(&self, indexp: usize, indexc: usize) -> Result<(Vec<f64>, f64), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let ncomp = self.tqnosc()?;
        let mut stoi: Vec<f64> = vec![0.0; ncomp];
        let mut wmass = 0.0f64;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    stoi: &mut f64,
                    wmass: &mut f64,
                    errcode: &mut ChemAppInt,
                ),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut stoi[0], &mut wmass, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    stoi: &mut f64,
                    wmass: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut stoi[0], &mut wmass, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((stoi, wmass), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-CHARGE-OF-PHASE-CONSTITUENT.
    ///
    /// ChemApp's `VAL` output is DOUBLE PRECISION; using `f64` prevents the
    /// native routine from writing an eight-byte value into integer storage.
    #[named]
    pub fn tqchar(&self, indexp: usize, indexc: usize) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let mut charge = 0.0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    charge: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut charge, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    charge: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexc, &mut charge, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result(charge, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-INDEX-NUMBER-OF-SUBLATTICE-CONSTITUENT
    #[named]
    pub fn tqinlc(&self, name: &str, indexp: usize, indexl: usize) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexl);
        let cname: CString = CString::new(name)?;
        let mut errcode = 0;
        let mut indexc: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    name: *const u8,
                    name_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                cstring_character_input(&cname)?.1,
                &indexp,
                &indexl,
                &mut indexc,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    name: *const u8,
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    name_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cname)?.0,
                &indexp,
                &indexl,
                &mut indexc,
                &mut errcode,
                cstring_character_input(&cname)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(indexc, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NAME-OF-SUBLATTICE-CONSTITUENT
    #[named]
    pub fn tqgnlc(
        &self,
        indexp: usize,
        indexl: usize,
        indexc: usize,
    ) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexl, indexc);
        let mut cname: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cname: &mut u8,
                    cname_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexl,
                &indexc,
                &mut cname[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cname: &mut u8,
                    errcode: &mut ChemAppInt,
                    cname_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexl,
                &indexc,
                &mut cname[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cname, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NUMBER-OF-SUBLATTICES
    #[named]
    pub fn tqnosl(&self, indexp: usize) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut errcode = 0;
        let mut nosl: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    nosl: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &mut nosl, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    nosl: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &mut nosl, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(nosl, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-NUMBER-OF-SUBLATTICE-SPECIES
    #[named]
    pub fn tqnolc(&self, indexp: usize, index: usize) -> Result<usize, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, index);
        let mut errcode = 0;
        let mut nosc = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    index: &ChemAppInt,
                    nosc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &index, &mut nosc, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    index: &ChemAppInt,
                    nosc: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &index, &mut nosc, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_nonnegative_result(nosc, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-STATUS-OF-PHASE
    #[named]
    pub fn tqgsp(&self, indexp: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut cstatus: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    cstatus: &mut u8,
                    cstatus_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cstatus[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    cstatus: &mut u8,
                    errcode: &mut ChemAppInt,
                    cstatus_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &mut cstatus[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cstatus, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-STATUS-OF-PHASE
    #[named]
    pub fn tqcsp(&self, indexp: usize, status: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let cstatus: CString = CString::new(status)?;
        let cstatus_length = cstring_character_input(&cstatus)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    cstatus: *const u8,
                    cstatus_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&cstatus)?.0,
                cstatus_length,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    cstatus: *const u8,
                    errcode: &mut ChemAppInt,
                    cstatus_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&cstatus)?.0,
                &mut errcode,
                cstatus_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-STATUS-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqgspc(&self, indexp: usize, indexc: usize) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let mut cstatus: [u8; NAME_LENGTH_MAX] = [0; NAME_LENGTH_MAX];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            // TQGSPC writes the fixed-width status buffer; express that write in
            // the Rust FFI type even though pointer representation is unchanged.
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cstatus: &mut u8,
                    cstatus_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                &mut cstatus[0],
                NAME_NATIVE_CHARACTER_LENGTH,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    cstatus: &mut u8,
                    errcode: &mut ChemAppInt,
                    cstatus_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                &mut cstatus[0],
                &mut errcode,
                NAME_NATIVE_CHARACTER_LENGTH,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fixed_fortran_string(&cstatus, NAME_LENGTH_MAX)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-STATUS-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqcspc(&self, indexp: usize, indexc: usize, status: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let cstatus: CString = CString::new(status)?;
        let cstatus_length = cstring_character_input(&cstatus)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    status: *const u8,
                    status_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                cstring_character_input(&cstatus)?.0,
                cstatus_length,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    status: *const u8,
                    errcode: &mut ChemAppInt,
                    status_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                cstring_character_input(&cstatus)?.0,
                &mut errcode,
                cstatus_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SET-EQUILIBRIUM-CONDITION
    #[named]
    pub fn tqsetc(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        val: f64,
    ) -> Result<i32, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let mut numcon: ChemAppInt = 0;
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    val: &f64,
                    numcon: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &val,
                &mut numcon,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    val: &f64,
                    numcon: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &val,
                &mut numcon,
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        return chemapp_int_to_i32(numcon);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// REMOVE-EQUILIBRIUM-CONDITION
    #[named]
    pub fn tqremc(&self, numcon: i32) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let numcon = i32_to_chemapp_int(numcon)?;
        let mut errcode: ChemAppInt = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(numcon: &ChemAppInt, errcode: &mut ChemAppInt) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&numcon, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(numcon: &ChemAppInt, errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&numcon, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SET-NAME-TEMPERATURE-PRESSURE-FOR-A-STREAM
    #[named]
    pub fn tqsttp(&self, idents: &str, vals: (f64, f64)) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        let vals_ = [vals.0, vals.1];
        let cidents: CString = CString::new(idents)?;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    idents: *const u8,
                    idents_len: ChemAppLen,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                cstring_character_input(&cidents)?.1,
                &vals_[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    idents: *const u8,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                    idents_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                &vals_[0],
                &mut errcode,
                cstring_character_input(&cidents)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SET-CONSTITUENT-AMOUNTS-FOR-A-STREAM
    #[named]
    pub fn tqstca(
        &self,
        idents: &str,
        indexp: usize,
        indexc: usize,
        val: f64,
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let cidents: CString = CString::new(idents)?;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    idents: *const u8,
                    idents_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                cstring_character_input(&cidents)?.1,
                &indexp,
                &indexc,
                &val,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    idents: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                    idents_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                &indexp,
                &indexc,
                &val,
                &mut errcode,
                cstring_character_input(&cidents)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SET-EQUILIBRIUM-CONDITION-WHEN-STREAM-INPUT
    #[named]
    pub fn tqstec(&self, option: &str, indexp: usize, val: f64) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let coption: CString = CString::new(option)?;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexp,
                &val,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &val,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// REMOVE-STREAM
    #[named]
    pub fn tqstrm(&self, idents: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let cidents: CString = CString::new(idents)?;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    idents: *const u8,
                    idents_len: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                cstring_character_input(&cidents)?.1,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    idents: *const u8,
                    errcode: &mut ChemAppInt,
                    idents_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                &mut errcode,
                cstring_character_input(&cidents)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-EQUILIBRIUM
    #[named]
    pub fn tqce(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let vals_: [f64; 2] = [vals.0, vals.1];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-EQUILIBRIUM-AND-LIST-RESULTS
    #[named]
    pub fn tqcel(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let vals_: [f64; 2] = [vals.0, vals.1];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS
    #[named]
    pub fn tqcen(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let vals_: [f64; 2] = [vals.0, vals.1];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-EQUILIBRIUM-FROM-PREVIOUS-AND-LIST-RESULTS
    #[named]
    pub fn tqcenl(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let vals_: [f64; 2] = [vals.0, vals.1];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP
    ///
    /// `ICONT` is a signed ChemApp INTEGER. Positive values request another
    /// continuation call; zero or negative values terminate the map.
    #[named]
    pub fn tqmap(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<i32, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let mut errcode = 0;
        let mut icont: ChemAppInt = 0;
        let vals_: [f64; 2] = [vals.0, vals.1];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    icont: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexp,
                &indexc,
                &vals_[0],
                &mut icont,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    icont: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut icont,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result(icont, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CALCULATE-ONE-DIMENSIONAL-PHASE-MAP-AND-LIST-RESULTS
    ///
    /// The signed continuation result has the same semantics as `tqmap`.
    #[named]
    pub fn tqmapl(
        &self,
        option: &str,
        indexp: usize,
        indexc: usize,
        vals: (f64, f64),
    ) -> Result<i32, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let mut errcode = 0;
        let mut icont: ChemAppInt = 0;
        let vals_: [f64; 2] = [vals.0, vals.1];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    icont: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexp,
                &indexc,
                &vals_[0],
                &mut icont,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    vals: &f64,
                    icont: &mut ChemAppInt,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &vals_[0],
                &mut icont,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result(icont, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGE-LIMIT-OF-TARGET-VARIABLE
    #[named]
    pub fn tqclim(&self, option: &str, val: f64) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &val,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &val,
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// SHOW-PRESENT-SETTINGS
    #[named]
    pub fn tqshow(&self) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<extern "system" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<extern "C" fn(errcode: &mut ChemAppInt) -> ()> =
                self.library.get(fname.as_bytes())?;
            func(&mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-RESULT for the scalar subset of `TQGETR`.
    ///
    /// This wrapper deliberately exposes only selector forms that produce one
    /// `DOUBLE PRECISION` result: a positive phase and positive constituent or
    /// system-component index; a positive phase with index zero; system
    /// component index with `indexp == 0`; and the whole-system form `(0, 0)`.
    /// The Programmer's Manual defines negative selectors that return arrays;
    /// they are not representable by this unsigned scalar API and therefore
    /// cannot reach its one-element native output buffer.
    #[named]
    pub fn tqgetr(&self, option: &str, indexp: usize, indexc: usize) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc);
        let coption: CString = CString::new(option)?;
        let option_length = cstring_character_input(&coption)?.1;
        let mut value = 0.0f64;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_length: ChemAppLen,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    value: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                option_length,
                &indexp,
                &indexc,
                &mut value,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    value: &mut f64,
                    errcode: &mut ChemAppInt,
                    option_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &indexc,
                &mut value,
                &mut errcode,
                option_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result(value, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-PROPERTY-OF-A-PHASE-CONSTITUENT
    #[named]
    pub fn tqgdpc(&self, option: &str, indexp: usize, index: usize) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, index);
        let coption: CString = CString::new(option)?;
        let mut fval = 0.0f64;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexp: &ChemAppInt,
                    index: &ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexp,
                &index,
                &mut fval,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    option: *const u8,
                    indexp: &ChemAppInt,
                    index: &ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&coption)?.0,
                &indexp,
                &index,
                &mut fval,
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fval, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-THERMODYNAMIC-PROPERTY-OF-A-STREAM
    #[named]
    pub fn tqstxp(&self, idents: &str, option: &str) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        let cidents: CString = CString::new(idents)?;
        let coption: CString = CString::new(option)?;
        let mut errcode = 0;
        let mut fval = 0.0f64;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    idents: *const u8,
                    idents_len: ChemAppLen,
                    option: *const u8,
                    option_len: ChemAppLen,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                cstring_character_input(&cidents)?.1,
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &mut fval,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    idents: *const u8,
                    option: *const u8,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                    idents_len: ChemAppLen,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cidents)?.0,
                cstring_character_input(&coption)?.0,
                &mut fval,
                &mut errcode,
                cstring_character_input(&cidents)?.1,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        return wrap_result(fval, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-CALCULATED-EQUILIBRIUM-SUBLATTICE-SITE-FRACTION
    #[named]
    pub fn tqgtlc(&self, indexp: usize, indexl: usize, indexc: usize) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexl, indexc);
        let mut errcode = 0;
        let mut fval = 0.0f64;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexl, &indexc, &mut fval, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexl: &ChemAppInt,
                    indexc: &ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&indexp, &indexl, &indexc, &mut fval, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result(fval, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-CALCULATED-QUADRUPLET-OR-PAIR-FRACTION
    #[named]
    pub fn tqbond(
        &self,
        indexp: usize,
        indexa: usize,
        indexb: usize,
        indexc: usize,
        indexd: usize,
    ) -> Result<f64, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexa, indexb, indexc, indexd);
        let mut value = 0.0;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexa: &ChemAppInt,
                    indexb: &ChemAppInt,
                    indexc: &ChemAppInt,
                    indexd: &ChemAppInt,
                    value: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexa,
                &indexb,
                &indexc,
                &indexd,
                &mut value,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexa: &ChemAppInt,
                    indexb: &ChemAppInt,
                    indexc: &ChemAppInt,
                    indexd: &ChemAppInt,
                    value: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexa,
                &indexb,
                &indexc,
                &indexd,
                &mut value,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        return wrap_result(value, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-ERROR-MESSAGE
    #[named]
    pub fn tqerr(&self) -> Result<String, ChemAppError> {
        let fname = func_alias(function_name!());
        // TQERR returns three CHARACTER*80 records. The allocation is 240
        // bytes, but the hidden length remains the length of each record: 80.
        let mut cmess: [u8; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT] =
            [0; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT];
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    message: &mut u8,
                    message_len: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut cmess[0], TQERR_NATIVE_RECORD_LENGTH, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    message: &mut u8,
                    errcode: &mut ChemAppInt,
                    message_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&mut cmess[0], &mut errcode, TQERR_NATIVE_RECORD_LENGTH);
        }
        /******************************************************************************************************/
        return wrap_result(tqerr_message(&cmess)?, errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-INPUT-THERMODYNAMIC-DATA-OF-PHASE-CONSTITUENT
    #[named]
    pub fn tqgdat(
        &self,
        indexp: usize,
        indexc: usize,
        option: &str,
        indexr: usize,
    ) -> Result<Vec<f64>, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexc, indexr);
        let mut errcode = 0;
        let mut fval = [0.0; 25];
        let mut nfval: ChemAppInt = 0;
        let coption: CString = CString::new(option)?;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexr: &ChemAppInt,
                    nfval: &mut ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexr,
                &mut nfval,
                &mut fval[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    indexc: &ChemAppInt,
                    option: *const u8,
                    indexr: &ChemAppInt,
                    nfval: &mut ChemAppInt,
                    fval: &mut f64,
                    errcode: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                &indexc,
                cstring_character_input(&coption)?.0,
                &indexr,
                &mut nfval,
                &mut fval[0],
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        let nfval = chemapp_int_to_usize(nfval)?;
        if nfval > fval.len() {
            return Err(ChemAppError::OtherError(format!(
                "TQGDAT returned {nfval} values for a {}-value buffer",
                fval.len()
            )));
        }
        return Ok(fval[0..nfval].to_vec());
    }

    /*****************************************************************************************************************************************************************************************************/
    /// LIST-EXCESS-PARAMETERS-OF-PHASE
    #[named]
    pub fn tqlpar(&self, indexp: usize, option: &str) -> Result<Vec<String>, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp);
        let mut errcode: ChemAppInt = 0;
        let mut nopar: ChemAppInt = 0;
        let coption: CString = CString::new(option)?;
        let dimensions = self.tqsize()?;
        let descriptor_capacity = match option.trim().to_ascii_uppercase().as_str() {
            "G" => usize::try_from(dimensions.excess_gibbs_coefficients_per_phase),
            "M" => usize::try_from(dimensions.excess_magnetic_coefficients_per_phase),
            // Preserve Engine fidelity for unknown options: ChemApp decides
            // validity, while the larger list capacity keeps storage sufficient.
            _ => usize::try_from(
                dimensions
                    .excess_gibbs_coefficients_per_phase
                    .max(dimensions.excess_magnetic_coefficients_per_phase),
            ),
        }
        .map_err(|_| {
            ChemAppError::OtherError("TQSIZE returned a negative TQLPAR capacity".to_owned())
        })?;
        if descriptor_capacity == 0 {
            return Err(ChemAppError::OtherError(
                "TQSIZE returned zero TQLPAR capacity".to_owned(),
            ));
        }
        let mut lgtpar = vec![0 as ChemAppInt; descriptor_capacity];
        let mut chrpar = vec![[0u8; 156]; descriptor_capacity];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    option: *const u8,
                    option_len: ChemAppLen,
                    nopar: &mut ChemAppInt,
                    chrpar: &mut u8,
                    chrpar_len: ChemAppLen,
                    lgtpar: &mut ChemAppInt,
                    noerr: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &mut nopar,
                &mut chrpar[0][0],
                TQLPAR_NATIVE_RECORD_LENGTH,
                &mut lgtpar[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    option: *const u8,
                    nopar: &mut ChemAppInt,
                    chrpar: &mut u8,
                    lgtpar: &mut ChemAppInt,
                    noerr: &mut ChemAppInt,
                    option_len: ChemAppLen,
                    chrpar_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&coption)?.0,
                &mut nopar,
                &mut chrpar[0][0],
                &mut lgtpar[0],
                &mut errcode,
                cstring_character_input(&coption)?.1,
                TQLPAR_NATIVE_RECORD_LENGTH,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        let nopar = chemapp_int_to_usize(nopar)?;
        if nopar > chrpar.len() {
            return Err(ChemAppError::OtherError(format!(
                "TQLPAR returned {nopar} parameters for a {}-parameter buffer",
                chrpar.len()
            )));
        }
        let mut descriptors = Vec::with_capacity(nopar);
        for (record, raw_length) in chrpar.iter().zip(lgtpar.iter()).take(nopar) {
            let length = chemapp_int_to_usize(*raw_length)?;
            if length > record.len() {
                return Err(ChemAppError::OtherError(format!(
                    "TQLPAR returned record length {length} for a {}-byte record",
                    record.len()
                )));
            }
            descriptors.push(fixed_fortran_string(record, length)?);
        }
        return Ok(descriptors);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// GET-EXCESS-PARAMETERS-OF-PHASE
    #[named]
    pub fn tqgpar(
        &self,
        indexp: usize,
        option: &str,
        indexx: usize,
    ) -> Result<Vec<Vec<f64>>, ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(indexp, indexx);
        let mut errcode: ChemAppInt = 0;
        let coption: CString = CString::new(option)?;
        let mut noexpr: ChemAppInt = 0;
        let mut nvala: ChemAppInt = 0;
        // Fortran stores VALA(expression, value) column-major. The manual ties
        // its first/leading dimension to TQSIZE.NI, not returned NOEXPR.
        let leading_dimension =
            usize::try_from(self.tqsize()?.equations_per_constituent).map_err(|_| {
                ChemAppError::OtherError(
                    "TQSIZE returned a negative TQGPAR leading dimension".to_owned(),
                )
            })?;
        if leading_dimension == 0 {
            return Err(ChemAppError::OtherError(
                "TQSIZE returned zero TQGPAR leading dimension".to_owned(),
            ));
        }
        let raw_capacity = leading_dimension
            .checked_mul(TQGPAR_VALUE_CAPACITY)
            .ok_or_else(|| {
                ChemAppError::OtherError("TQGPAR buffer extent overflowed usize".to_owned())
            })?;
        let mut vala = vec![0.0; raw_capacity];
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    indexp: &ChemAppInt,
                    option: *const u8,
                    option_len: ChemAppLen,
                    indexx: &ChemAppInt,
                    noexpr: &mut ChemAppInt,
                    nvala: &mut ChemAppInt,
                    vala: &mut f64,
                    noerr: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&coption)?.0,
                cstring_character_input(&coption)?.1,
                &indexx,
                &mut noexpr,
                &mut nvala,
                &mut vala[0],
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    indexp: &ChemAppInt,
                    option: *const u8,
                    indexx: &ChemAppInt,
                    noexpr: &mut ChemAppInt,
                    nvala: &mut ChemAppInt,
                    vala: &mut f64,
                    noerr: &mut ChemAppInt,
                    option_len: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                &indexp,
                cstring_character_input(&coption)?.0,
                &indexx,
                &mut noexpr,
                &mut nvala,
                &mut vala[0],
                &mut errcode,
                cstring_character_input(&coption)?.1,
            );
        }
        /******************************************************************************************************/
        wrap_result((), errcode)?;
        let noexpr = chemapp_int_to_usize(noexpr)?;
        let nvala = chemapp_int_to_usize(nvala)?;
        return tqgpar_values(&vala, leading_dimension, noexpr, nvala);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// CHANGES-DATA-OF-THERMODYNAMIC-DATA-FILE
    #[named]
    pub fn tqcdat(
        &self,
        i1: usize,
        i2: usize,
        i3: usize,
        i4: usize,
        i5: usize,
        val: f64,
    ) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        raw_chemapp_ints!(i1, i2, i3, i4, i5);
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    i1: &ChemAppInt,
                    i2: &ChemAppInt,
                    i3: &ChemAppInt,
                    i4: &ChemAppInt,
                    i5: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&i1, &i2, &i3, &i4, &i5, &val, &mut errcode);
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    i1: &ChemAppInt,
                    i2: &ChemAppInt,
                    i3: &ChemAppInt,
                    i4: &ChemAppInt,
                    i5: &ChemAppInt,
                    val: &f64,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(&i1, &i2, &i3, &i4, &i5, &val, &mut errcode);
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }

    /*****************************************************************************************************************************************************************************************************/
    /// WRITE-DATA-FILE-IN-ASCII-FORMAT
    #[named]
    pub fn tqwasc(&self, file: &str) -> Result<(), ChemAppError> {
        let fname = func_alias(function_name!());
        let cfile = CString::new(file)?;
        let cfile_length = cstring_character_input(&cfile)?.1;
        let mut errcode = 0;
        /******************************************************************************************************/
        #[cfg(target_family = "windows")]
        unsafe {
            let func: Symbol<
                extern "system" fn(
                    cfile: *const u8,
                    cfile_length: ChemAppLen,
                    errcode: &mut ChemAppInt,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cfile)?.0,
                cfile_length,
                &mut errcode,
            );
        }
        /******************************************************************************************************/
        #[cfg(target_family = "unix")]
        unsafe {
            let func: Symbol<
                extern "C" fn(
                    cfile: *const u8,
                    errcode: &mut ChemAppInt,
                    cfile_length: ChemAppLen,
                ) -> (),
            > = self.library.get(fname.as_bytes())?;
            func(
                cstring_character_input(&cfile)?.0,
                &mut errcode,
                cfile_length,
            );
        }
        /******************************************************************************************************/
        return wrap_result((), errcode);
    }
}

/***********************************************************************************************************************************************************************************************************/
/***********************************************************************************************************************************************************************************************************/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_ownership_can_move_between_threads() {
        fn assert_send<T: Send>() {}
        assert_send::<Engine>();
    }

    #[test]
    fn fixed_output_lengths_match_the_checked_gtt_bridge() {
        assert_eq!(TQGTID_CHARACTER_LENGTH, 255);
        assert_eq!(TQGTNM_CHARACTER_LENGTH, 80);
        assert_eq!(NAME_LENGTH_MAX, 25);
        assert_eq!(TQGTRH_PROGRAM_NAME_LENGTH, 40);
        assert_eq!(TQGTRH_USER_ID_LENGTH, 255);
        assert_eq!(TQGTRH_TEXT_LENGTH, 80);
        assert_eq!(TQERR_RECORD_LENGTH, 80);
        assert_eq!(TQERR_RECORD_COUNT, 3);
    }

    #[test]
    fn fixed_fortran_string_preserves_internal_spaces_and_trims_only_padding() {
        assert_eq!(
            fixed_fortran_string(b"GTT - Technologies      ", 24).unwrap(),
            "GTT - Technologies"
        );
        assert_eq!(
            fixed_fortran_string(b"Corundum                 ", 25).unwrap(),
            "Corundum"
        );
        assert_eq!(fixed_fortran_string(b"    ", 4).unwrap(), "");
        assert_eq!(
            fixed_fortran_string(b"A B   \0not-a-record", 6).unwrap(),
            "A B"
        );
        assert!(fixed_fortran_string(b"short", 6).is_err());
    }

    #[test]
    fn tqgtnm_fixed_record_preserves_complete_license_holder_text() {
        let license_holder = b"Example Research License - University";
        let mut record = [b' '; TQGTNM_CHARACTER_LENGTH];
        record[..license_holder.len()].copy_from_slice(license_holder);
        assert_eq!(
            fixed_fortran_string(&record, TQGTNM_CHARACTER_LENGTH).unwrap(),
            "Example Research License - University"
        );
    }

    #[test]
    fn tqerr_is_three_fixed_width_records() {
        let mut buffer = [b' '; TQERR_RECORD_LENGTH * TQERR_RECORD_COUNT];
        buffer[..15].copy_from_slice(b"Example Program");
        let second = b"Copyright Example Organization, 100 Research Road, Example City";
        buffer[TQERR_RECORD_LENGTH..TQERR_RECORD_LENGTH + second.len()].copy_from_slice(second);
        let third = b"https://example.invalid";
        buffer[2 * TQERR_RECORD_LENGTH..2 * TQERR_RECORD_LENGTH + third.len()]
            .copy_from_slice(third);
        assert_eq!(
			tqerr_message(&buffer).unwrap(),
			"Example Program\nCopyright Example Organization, 100 Research Road, Example City\nhttps://example.invalid"
		);
    }

    #[test]
    fn tqchar_exposes_the_native_double_precision_result() {
        let _: fn(&Engine, usize, usize) -> Result<f64, ChemAppError> = Engine::tqchar;
    }

    #[test]
    fn tqgpar_reconstructs_fortran_column_major_rows() {
        let leading_dimension = 5;
        let mut raw = vec![0.0; leading_dimension * 3];
        raw[0] = 10.0;
        raw[1] = 20.0;
        raw[leading_dimension] = 11.0;
        raw[leading_dimension + 1] = 21.0;
        raw[2 * leading_dimension] = 12.0;
        raw[2 * leading_dimension + 1] = 22.0;
        assert_eq!(
            tqgpar_values(&raw, leading_dimension, 2, 3).unwrap(),
            vec![vec![10.0, 11.0, 12.0], vec![20.0, 21.0, 22.0]]
        );
    }

    #[test]
    fn tqgpar_rejects_dimensions_outside_the_allocated_extent() {
        let raw = vec![0.0; 5 * TQGPAR_VALUE_CAPACITY];
        assert!(tqgpar_values(&raw, 5, 6, 1).is_err());
        assert!(tqgpar_values(&raw, 5, 1, TQGPAR_VALUE_CAPACITY + 1).is_err());
        assert!(tqgpar_values(&raw[..4], 5, 1, 1).is_err());
    }

    #[test]
    fn mapping_continuation_remains_signed() {
        let _: fn(&Engine, &str, usize, usize, (f64, f64)) -> Result<i32, ChemAppError> =
            Engine::tqmap;
        let _: fn(&Engine, &str, usize, usize, (f64, f64)) -> Result<i32, ChemAppError> =
            Engine::tqmapl;
    }
}
