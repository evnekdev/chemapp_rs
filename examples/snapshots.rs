//! Capture owned results before later ChemApp calls change the live state.

mod common;

use chemapp_rs::ChemAppError;

fn main() -> Result<(), ChemAppError> {
    let calculator = common::calculator_from_env()?;
    let composition = common::unit_component_composition(&calculator)?;
    let first_temperature = common::temperature()?;

    calculator.calculate_isothermal(&composition, first_temperature)?;
    let first = calculator.system().snapshot()?;
    let retained = first.clone();

    // The live entity now reflects the second state. Both owned clones retain
    // the earlier equilibrium and implement Debug for direct inspection.
    calculator.calculate_isothermal(&composition, first_temperature + 100.0)?;
    let second = calculator.system().snapshot()?;

    println!("First snapshot: {retained:?}");
    println!("Second snapshot: {second:?}");
    assert_eq!(first, retained);
    Ok(())
}
