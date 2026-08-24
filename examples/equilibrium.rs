//! Perform a first isothermal equilibrium in the native component basis.

mod common;

use chemapp_rs::ChemAppError;

fn main() -> Result<(), ChemAppError> {
    let calculator = common::calculator_from_env()?;
    let composition = common::unit_component_composition(&calculator)?;
    let temperature = common::temperature()?;
    let pressure = common::pressure()?;

    // This pedagogical composition is not scientifically meaningful for every
    // system. Replace it with the amounts required by your thermodynamic case.
    calculator.calculate_isothermal_at_pressure(&composition, temperature, pressure)?;

    let temperature_unit = calculator.engine().tqgsu("Temperature")?;
    let pressure_unit = calculator.engine().tqgsu("Pressure")?;
    println!("Equilibrium at {temperature} {temperature_unit}, {pressure} {pressure_unit}");
    println!("Stable phases (strict AC > 0.9999):");
    for phase in calculator.phases()? {
        if phase.is_stable()? {
            println!("  {}: amount = {:.8e}", phase.name()?, phase.a()?);
        }
    }
    Ok(())
}
