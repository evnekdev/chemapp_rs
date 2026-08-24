//! Prints every Gibbs and magnetic interaction in a loaded multicomponent DAT
//! model, including native TQLPAR text, name-resolved structure, live TQGPAR
//! values, and descriptor provenance.

mod common;

use chemapp_rs::{Calculator, ChemAppError};

fn main() -> Result<(), ChemAppError> {
    let calculator: Calculator = common::calculator_from_env()?;
    let phase_filter = std::env::var("CHEMAPP_PHASE").ok();

    eprintln!(
        "Descriptor cross-check: not configured (native TQLPAR parsing only; use \
         interaction_report_with_cross_check for a compatible ASCII DAT provider)."
    );

    for report in calculator.interaction_report()? {
        if phase_filter
            .as_deref()
            .is_some_and(|name| name != report.phase_name)
        {
            continue;
        }
        println!(
            "Phase {} [{}], model {}, {} sublattice(s): {} Gibbs, {} magnetic",
            report.phase_name,
            report.phase_index,
            report.model,
            report.sublattice_count,
            report.gibbs.len(),
            report.magnetic.len()
        );
        if !report.gibbs.is_empty() || !report.magnetic.is_empty() {
            println!("{}", report.table_string());
        }
    }
    Ok(())
}
