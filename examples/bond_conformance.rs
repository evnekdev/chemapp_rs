//! Opt-in live comparison of canonical bond entities, snapshots and raw selectors.
//! Usage: bond_conformance LIBRARY DATAFILE PHASE T_K FORMULA AMOUNT [...].
//! Only the selected phase is entered. Formula amounts prescribe incoming bulk,
//! not individual final site fractions. Native setup/query errors propagate.
use chemapp_rs::entities::phase::Phase;
use chemapp_rs::Calculator;
use std::error::Error;

/// Runs one isolated equilibrium and checks every canonical bond representation.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 6 || (args.len() - 4) % 2 != 0 {
        return Err(
            "usage: bond_conformance LIBRARY DATAFILE PHASE T_K FORMULA AMOUNT [...]".into(),
        );
    }
    let calculator = Calculator::from_library(&args[0], &args[1])?;
    let e = calculator.engine();
    let p = e.tqinp(&args[2])?;
    e.tqremc(-2)?;
    for j in 1..=e.tqnop()? {
        e.tqcsp(j, if j == p { "ENTERED" } else { "ELIMINATED" })?;
    }
    for (q, u) in [("T", "K"), ("P", "bar"), ("A", "mol"), ("E", "J")] {
        e.tqcsu(q, u)?;
    }
    e.tqsetc("T", 0, 0, args[3].parse()?)?;
    e.tqsetc("P", 0, 0, 1.)?;
    for pair in args[4..].chunks_exact(2) {
        let c = e.tqinpc(p, &pair[0])?;
        if !e.tqpcis(p, c)? {
            return Err("incoming constituent disallowed".into());
        }
        e.tqsetc("IA", p, c, pair[1].parse()?)?;
    }
    e.tqce(" ", 0, 0, (0., 0.))?;
    let phase = Phase::new(&calculator, p);
    let mut sum: f64 = 0.;
    let mut count = 0;
    for bond in phase.bonds()? {
        if !bond.is_valid()? {
            return Err("enumerated bond rejected by entity validation".into());
        }
        let value = bond.x()?;
        if !value.is_finite() || !(0. ..=1.).contains(&value) {
            return Err("bond fraction outside physical range".into());
        }
        if let Some(members) = bond.quadruplet_members() {
            let first = e.tqnolc(p, 1)?;
            let a = members[0].local_index();
            let b = members[1].local_index();
            let c = first + members[2].local_index();
            let d = first + members[3].local_index();
            if (e.tqbond(p, b, a, d, c)? - value).abs() > 1e-12 {
                return Err("raw within-sublattice permutation disagrees".into());
            }
        }
        let snapshot = bond.snapshot()?;
        if (snapshot.x - value).abs() > 1e-12 {
            return Err("snapshot differs".into());
        }
        sum += value;
        count += 1;
    }
    if count == 0 || (sum - 1.).abs() > 1e-10 {
        return Err("empty or unnormalized bond table".into());
    }
    println!(
        "version={} phase={} model={} bonds={count} fraction_sum={sum:.17}",
        e.tqvers()?,
        e.tqgnp(p)?,
        e.tqmodl(p)?
    );
    Ok(())
}
