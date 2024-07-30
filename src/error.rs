use std::fmt;
use lazy_static::{lazy_static};
use std::collections::{HashMap};
use std::str::{Utf8Error};
use std::ffi::{NulError};


/// Custom error struct
#[derive(Debug)]
pub enum ChemAppError{
	NativeError(usize),
	OtherError(String),
	CustomError(String),
}

impl ChemAppError {
	
	pub fn description(&self)->String{
		match self {
			Self::NativeError(id) => {
				match error_descriptions.get(&id) {
				Some(desc) => {return format!("ChemApp error {}, {}", id, desc);}
				None => {return format!("Unrecognized ChemApp error {}", id);}
			}
			}
			Self::OtherError(desc) => {return format!("{}", &desc);}
			Self::CustomError(desc)=> {return format!("{}", &desc);}
		}
	}
}

impl fmt::Display for ChemAppError {
	fn fmt(&self, formatter: &mut fmt::Formatter)->fmt::Result {
		write!(formatter, "{}", self.description());
		return Ok(());
	}
}

impl From<Utf8Error> for ChemAppError {
	fn from(error: Utf8Error)->ChemAppError {
		return ChemAppError::OtherError(error.to_string());
	}
}

impl From<NulError> for ChemAppError {
	fn from(error: NulError)->ChemAppError {
		return ChemAppError::OtherError(error.to_string());
	}
}

impl From<libloading::Error> for ChemAppError {
	fn from(error: libloading::Error)->ChemAppError {
		return ChemAppError::OtherError(error.to_string());
	}
}

lazy_static!{
	pub static ref error_descriptions : HashMap<usize,&'static str> = HashMap::from(
						[
										(0,"No error"),
										// initialization errors
                                        (101, "The subroutine 'TQINI' must be called first"),
										(102, "The input/output file has not been opened"),
										(103, "The thermodynamic data-file could not be completely entered due to a read error"),
										(104, "A thermodynamic data-file must be read first"),
										(105, "The entered unit number is not permitted (expected value: <=10 or >=20)"),
										(106, "The entered language index is out of range"),
										(107, "The file cannot be opened"),
										(108, "The file cannot be closed"),
										(109, "The character string could not be written to the unit"),
										(110, "A phase or phase constituent could not be completely entered due to a read error"),
										(111, "Excess magnetic interaction could not be completely entered"),
										(112, "Excess Gibbs energy interaction could not be completely entered"),
										(150, "A transparent data-file must be read first"),
										(151, "The user ID is corrupt"),
										(152, "The user name is corrupt"),
										(153, "No proper user ID authorization to enter the data-file"),
										(154, "This version of ChemApp cannot write any data-files"),
										(155, "The checksum calculation indicates that the data-file is corrupt"),
										(156, "The size of the thermochemical system is different from what is specified in the header"),
										(157, "The thermodynamic data-file has reached its expiry date"),
										(158, "The program/library name is corrupt"),
										(159, "This program is not authorized to read this data-file"),
										(160, "The thermochemical system is too big for this version of ChemApp"),
										(161, "The magic bytes indicate that the file entered is not a transparent data-file"),
										(162, "The transparent file format version is unknown to this version of ChemApp"),
										(163, "No valid dongle found"),
										(164, "Dongle licensing information does not authorize the execution of this program"),
										// errors in entered character options
										(201, "The entered option cannot be interpreted"),
										(202, "The entered stream identifier cannot be interpreted"),
										(203, "The entered option is not implemented"),
										(204, "Amount unit reset to mol (the molecular mass of a phase constituent is <=0)"),
										// errors in entered names of components, phases, or constituents
										(301, "The character input contains more than 24 characters"),
										(302, "The character input is not uniquely abbreviated"),
										(303, "The character input is not a phase"),
										(304, "The character input is not a constituent of the entered phase"),
										(305, "The character input is not a system component"),
										(306, "The character input is not a phase constituent or a system component"),
										(307, "Names for a number of system components that is equal to that of the thermodynamic data-file have to be entered"),
										(308, "The entered system components are not linearly independent"),
										(309, "Status for the selected phase constituent cannot be changed"),
										(310, "Statuses for the constituents of a phase are inconsistent"),
										(311, "The character input is not a constituent of the entered sublattice"),
										(312, "<NAME> is a system component, but without charge"),
										// errors in entered index numbers
										(401, "The entered system component index is out of range"),
										(402, "The entered phase index is out of range"),
										(403, "The entered phase constituent index is out of range"),
										(404, "The entered value of the variable NUMCON is false"),
										(405, "The total number of streams or stream constituents is out of range"),
										(406, "The entered sublattice index is out of range"),
										(407, "The entered sublattice constituent index is out of range"),
										// errors in setting conditions
										(501, "'TQSETC' and 'TQSTCA' cannot be used interchangeably for setting conditions"),
										(502, "The entered option is not defined for system components"),
										(503, "The entered constituent amounts for streams must not be less than zero"),
										(504, "The entered option is not permitted, please use 'TQSTCA' or 'TQCE'"),
										(505, "Enter the lower and upper limits of the target variable in reverse order"),
										(506, "The lower and upper limits (VALS(1) and VALS(2)) must differ"),
										(507, "The upper limit (VALS(2)) must be greater than (or equal to) zero"),
										(508, "Enter input composition before executing the equilibrium calculation"),
										(509, "Enter a pressure > 0 before executing the equilibrium calculation"),
										(510, "Enter a temperature > 0 K before executing the equilibrium calculation"),
										(511, "Define a target before executing the equilibrium calculation"),
										(512, "Define a target variable before executing the equilibrium calculation"),
										(513, "Modify input composition before executing the equilibrium calculation"),
										(514, "Incoming amounts for metallic constituents of phases described by the two-sublattice ionic formalism and for constituents of phases described by the species chemical potential/bond energy formalism cannot be entered"),
										(515, "Target calculations are not permitted with ChemApp 'light'"),
										(516, "Calculate a chemical equilibrium with 'TQCE' before calling 'TQCEN'"),
										// errors when getting results
										(601, "Incoming amounts are not calculated for phases or the entire system"),
										(602, "Activities for the entire system are not defined"),
										(603, "Extensive properties for system components are not defined"),
										(604, "Fractions of system components are not permitted for phase indices <= 0"),
										(605, "Fractions of system components cannot be calculated"),
										(606, "Activities of system components cannot be calculated in this case"),
										(607, "Mole fractions of pairs or quadruplets are not calculated"),
										(608, "Extensive properties are not calculated when stream constituents are eliminated"),
										(609, "Eh or pH cannot be calculated"),
										// errors when calculating the chemical equilibrium composition
										(701, "Equilibrium composition not obtained; all possible assemblies of phases were considered"),
										(702, "Equilibrium composition not obtained; 200 different assemblies of phases were considered"),
										(703, "Equilibrium composition not obtained; the reactant input cannot correspond to chemical equilibrium conditions"),
										(704, "Equilibrium composition not obtained; one reactant amount must be independent and different from zero"),
										(705, "Equilibrium composition not obtained; the mass balance equations cannot be solved"),
										(706, "Equilibrium composition not obtained; this constant volume calculation cannot be executed"),
										(707, "Target calculation aborted; the maximum number of iterations (99) is exceeded"),
										(708, "Target calculation aborted; reactant activities are not permitted"),
										(709, "Target calculation aborted; negative reactant amounts are not permitted"),
										(710, "Target calculation aborted; the entered phase cannot be target phase under the given conditions"),
										(711, "Target calculation aborted; no solution is found within the permitted interval"),
										(712, "Target calculation aborted; the value of the target variable is less than the lowest permitted"),
										(713, "Target calculation aborted; the value of the target variable is greater than the highest permitted"),
										// errors returned from subroutines interfacing ChemApp to languages other than Fortran
										(901, "The file cannot be opened"), // (changed to error no. 107 as of ChemApp V4.0.0)
										(902, "The file cannot be closed"), // (changed to error no. 108 as of ChemApp V4.0.0)
										// errors in custom subroutines
										(1000, "An ASCII data-file must be read first"),
										(1003, "The entered index number for a Cp equation is out of range"),
										(1004, "No molar volume data have been entered for the requested phase"),
										(1005, "No real gas data have been entered for the requested phase"),
										(1006, "No magnetic data have been entered for the requested phase"),
										(1007, "The requested phase is not aqueous"),
										(1008, "No excess Gibbs energy data have been entered for the requested phase"),
										(1009, "No excess magnetic data have been entered for the requested phase"),
										(1021, "The first entered index number is out of range"),
										(1022, "The second entered index number is out of range"),
										(1023, "The third entered index number is out of range"),
										(1024, "The fourth entered index number is out of range"),
										(1025, "No equation-of-state terms have been entered for the requested phase")
					  ]);
}
					  