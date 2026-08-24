//! ADVANCED: reversibly change one runtime-verified interaction parameter.
//!
//! TQCDAT mutates the loaded in-memory model. Existing equilibrium results are
//! stale after a write. This example always attempts to restore the captured
//! baseline and never writes a DAT file.

mod common;

use chemapp_rs::{ChemAppError, InteractionMutationSupport};

fn main() -> Result<(), ChemAppError> {
    let calculator = common::calculator_from_env()?;
    let reports = calculator.interaction_report()?;
    let address = reports
        .iter()
        .flat_map(|report| report.gibbs.iter().chain(&report.magnetic))
        .flat_map(|interaction| interaction.parameter_cells())
        .find_map(|cell| match cell.mutation {
            InteractionMutationSupport::Verified(address) => Some(address),
            InteractionMutationSupport::ReadOnly { .. } => None,
        })
        .ok_or_else(|| {
            ChemAppError::OtherError(
                "the loaded system has no runtime-verified mutable interaction cell".to_owned(),
            )
        })?;

    let baseline = calculator.interaction_parameter(address)?.value;
    let trial = if baseline == 0.0 {
        f64::EPSILON
    } else {
        baseline + baseline.abs() * 1.0e-9
    };

    let mutation_result = (|| -> Result<f64, ChemAppError> {
        calculator.set_interaction_parameter(address, trial)?;
        Ok(calculator.interaction_parameter(address)?.value)
    })();
    let restore_result = calculator.set_interaction_parameter(address, baseline);

    match (mutation_result, restore_result) {
        (Ok(observed), Ok(())) => {
            println!("Address: {address:?}");
            println!("Baseline: {baseline:.8e}");
            println!("Temporary value: {observed:.8e}");
            println!("Baseline restored; recalculate before reading equilibrium results.");
            Ok(())
        }
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(restore)) => Err(restore),
        (Err(primary), Err(restore)) => Err(ChemAppError::CleanupError {
            operation: "temporary interaction-parameter mutation".to_owned(),
            primary: Box::new(primary),
            cleanup: Box::new(restore),
        }),
    }
}
