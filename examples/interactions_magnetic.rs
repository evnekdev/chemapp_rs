mod common;

use chemapp_rs::ChemAppError;

fn main() -> Result<(), ChemAppError> {
    let calculator = common::calculator_from_env()?;

    for mut report in calculator.interaction_report()? {
        report.gibbs.clear();
        if !report.magnetic.is_empty() {
            println!("{}", report.table_string());
        }
    }
    Ok(())
}
