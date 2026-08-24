//! Load a ChemApp system and inspect its components and phases without
//! assuming dataset-specific names.

mod common;

use chemapp_rs::ChemAppError;

fn main() -> Result<(), ChemAppError> {
    let calculator = common::calculator_from_env()?;

    println!("ChemApp version: {}", calculator.engine().tqvers()?);
    println!("\nSystem components");
    println!("{:<7} Name", "Index");
    for component in calculator.components()? {
        println!("{:<7} {}", component.index(), component.name()?);
    }

    println!("\nPhases");
    println!("{:<7} {:<28} Model", "Index", "Name");
    for phase in calculator.phases()? {
        println!(
            "{:<7} {:<28} {}",
            phase.index(),
            phase.name()?,
            phase.model()?
        );
    }
    Ok(())
}
