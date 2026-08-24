//! Shared deterministic table rows for live entities and snapshots.

use comfy_table::{presets::UTF8_FULL_CONDENSED, Table};

use crate::entities::{
    bond::Bond, component::SystemComponent, constituent::Constituent, phase::Phase,
    species::Species, stream::Stream, system::System,
};
use crate::error::ChemAppError;
use crate::snapshot::{
    is_stable_phase_activity, BondSnapshot, BondSnapshotKind, CalculatorSnapshot,
    ConstituentSnapshot, PhaseSnapshot, SpeciesSnapshot, StreamSnapshot, SystemComponentSnapshot,
    SystemSnapshot, UnitsSnapshot,
};

fn number(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else {
        format!("{value:.8e}")
    }
}

fn vector(values: &[f64]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| number(*value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(crate) fn render(title: &str, headers: &[String], rows: Vec<Vec<String>>) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(headers.to_vec());
    for row in rows {
        table.add_row(row);
    }
    format!("{title}\n{table}")
}

struct SystemTableRow {
    values: [f64; 9],
}

impl From<&SystemSnapshot> for SystemTableRow {
    fn from(value: &SystemSnapshot) -> Self {
        Self {
            values: [
                value.t, value.p, value.a, value.vt, value.cp, value.h, value.s, value.g, value.v,
            ],
        }
    }
}

impl TryFrom<&System<'_>> for SystemTableRow {
    type Error = ChemAppError;
    fn try_from(value: &System<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            values: [
                value.t()?,
                value.p()?,
                value.a()?,
                value.vt()?,
                value.cp()?,
                value.h()?,
                value.s()?,
                value.g()?,
                value.v()?,
            ],
        })
    }
}

struct ComponentTableRow {
    index: usize,
    name: String,
    ia: f64,
    a: f64,
    x: f64,
    ac: f64,
    mu: f64,
    wmass: f64,
    stoic: Vec<f64>,
}

impl From<&SystemComponentSnapshot> for ComponentTableRow {
    fn from(v: &SystemComponentSnapshot) -> Self {
        Self {
            index: v.index,
            name: v.name.clone(),
            ia: v.ia,
            a: v.a,
            x: v.x,
            ac: v.ac,
            mu: v.mu,
            wmass: v.wmass,
            stoic: v.stoic.clone(),
        }
    }
}

impl TryFrom<&SystemComponent<'_>> for ComponentTableRow {
    type Error = ChemAppError;
    fn try_from(value: &SystemComponent<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            index: value.index(),
            name: value.name()?,
            ia: value.ia()?,
            a: value.a()?,
            x: value.x()?,
            ac: value.ac()?,
            mu: value.mu()?,
            wmass: value.wmass()?,
            stoic: value.stoic()?,
        })
    }
}

struct PhaseTableRow {
    index: usize,
    name: String,
    model: String,
    status: String,
    stable: bool,
    values: [f64; 13],
}

impl From<&PhaseSnapshot> for PhaseTableRow {
    fn from(v: &PhaseSnapshot) -> Self {
        Self {
            index: v.index,
            name: v.name.clone(),
            model: v.model.clone(),
            status: v.status.clone(),
            stable: is_stable_phase_activity(v.ac),
            values: [
                v.a, v.ac, v.mu, v.h, v.s, v.g, v.cp, v.v, v.hm, v.sm, v.gm, v.cpm, v.vm,
            ],
        }
    }
}

impl TryFrom<&Phase<'_>> for PhaseTableRow {
    type Error = ChemAppError;
    fn try_from(value: &Phase<'_>) -> Result<Self, Self::Error> {
        let ac = value.ac()?;
        Ok(Self {
            index: value.index(),
            name: value.name()?,
            model: value.model()?,
            status: value.status()?,
            stable: is_stable_phase_activity(ac),
            values: [
                value.a()?,
                ac,
                value.mu()?,
                value.h()?,
                value.s()?,
                value.g()?,
                value.cp()?,
                value.v()?,
                value.hm()?,
                value.sm()?,
                value.gm()?,
                value.cpm()?,
                value.vm()?,
            ],
        })
    }
}

struct ConstituentTableRow {
    phase: String,
    index: usize,
    name: String,
    status: String,
    incoming: bool,
    charge: f64,
    wmass: f64,
    values: [f64; 14],
    stoic: Vec<f64>,
}

impl From<&ConstituentSnapshot> for ConstituentTableRow {
    fn from(v: &ConstituentSnapshot) -> Self {
        Self {
            phase: format!("{} [{}]", v.phase_name, v.phase_index),
            index: v.index,
            name: v.name.clone(),
            status: v.status.clone(),
            incoming: v.incoming_allowed,
            charge: v.charge,
            wmass: v.wmass,
            values: [
                v.ia, v.a, v.ac, v.mu, v.h, v.s, v.g, v.cp, v.v, v.hm, v.sm, v.gm, v.cpm, v.vm,
            ],
            stoic: v.stoic.clone(),
        }
    }
}

impl TryFrom<&Constituent<'_>> for ConstituentTableRow {
    type Error = ChemAppError;
    fn try_from(value: &Constituent<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            phase: format!(
                "{} [{}]",
                value.calculator.engine.tqgnp(value.phase_index())?,
                value.phase_index()
            ),
            index: value.index(),
            name: value.name()?,
            status: value.status()?,
            incoming: value.incoming_allowed()?,
            charge: value.charge()?,
            wmass: value.wmass()?,
            values: [
                value.ia()?,
                value.a()?,
                value.ac()?,
                value.mu()?,
                value.h()?,
                value.s()?,
                value.g()?,
                value.cp()?,
                value.v()?,
                value.hm()?,
                value.sm()?,
                value.gm()?,
                value.cpm()?,
                value.vm()?,
            ],
            stoic: value.stoic()?,
        })
    }
}

struct SpeciesTableRow {
    phase: String,
    sublattice: usize,
    index: usize,
    name: String,
    x: f64,
}

impl From<&SpeciesSnapshot> for SpeciesTableRow {
    fn from(v: &SpeciesSnapshot) -> Self {
        Self {
            phase: format!("{} [{}]", v.phase_name, v.phase_index),
            sublattice: v.identity.sublattice,
            index: v.identity.local_index,
            name: v.name.clone(),
            x: v.x,
        }
    }
}

impl TryFrom<&Species<'_>> for SpeciesTableRow {
    type Error = ChemAppError;
    fn try_from(value: &Species<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            phase: format!(
                "{} [{}]",
                value.calculator.engine.tqgnp(value.phase_index())?,
                value.phase_index()
            ),
            sublattice: value.sublattice(),
            index: value.local_index(),
            name: value.name()?,
            x: value.x()?,
        })
    }
}

struct BondTableRow {
    phase: String,
    model: String,
    kind: &'static str,
    members: String,
    x: f64,
}

impl From<&BondSnapshot> for BondTableRow {
    fn from(v: &BondSnapshot) -> Self {
        let (kind, members) = match &v.kind {
            BondSnapshotKind::Pair {
                constituent_a,
                constituent_b,
            } => (
                "Pair",
                format!(
                    "{} [{}] - {} [{}]",
                    constituent_a.name,
                    constituent_a.constituent_index,
                    constituent_b.name,
                    constituent_b.constituent_index
                ),
            ),
            BondSnapshotKind::Quadruplet {
                species_a,
                species_b,
                species_c,
                species_d,
            } => (
                "Quadruplet",
                format!(
                    "{} [S1:{}], {} [S1:{}] | {} [S2:{}], {} [S2:{}]",
                    species_a.name,
                    species_a.identity.local_index,
                    species_b.name,
                    species_b.identity.local_index,
                    species_c.name,
                    species_c.identity.local_index,
                    species_d.name,
                    species_d.identity.local_index
                ),
            ),
        };
        Self {
            phase: format!("{} [{}]", v.phase_name, v.phase_index),
            model: v.model.clone(),
            kind,
            members,
            x: v.x,
        }
    }
}

impl TryFrom<&Bond<'_>> for BondTableRow {
    type Error = ChemAppError;
    fn try_from(value: &Bond<'_>) -> Result<Self, Self::Error> {
        Ok(Self::from(&value.snapshot()?))
    }
}

struct StreamTableRow {
    name: String,
    values: [f64; 7],
}

impl From<&StreamSnapshot> for StreamTableRow {
    fn from(v: &StreamSnapshot) -> Self {
        Self {
            name: v.name.clone(),
            values: [v.temperature, v.pressure, v.cp, v.h, v.s, v.g, v.v],
        }
    }
}

impl TryFrom<&Stream<'_>> for StreamTableRow {
    type Error = ChemAppError;
    fn try_from(value: &Stream<'_>) -> Result<Self, Self::Error> {
        Ok(Self::from(&value.snapshot()?))
    }
}

fn system_table(row: SystemTableRow, units: &UnitsSnapshot) -> String {
    let headers = [
        format!("T / {}", units.temperature),
        format!("P / {}", units.pressure),
        format!("A / {}", units.amount),
        format!("VT / {}", units.volume),
        format!("Cp / {}/K", units.energy),
        format!("H / {}", units.energy),
        format!("S / {}/K", units.energy),
        format!("G / {}", units.energy),
        format!("V / {}", units.volume),
    ];
    render(
        "System",
        &headers,
        vec![row.values.into_iter().map(number).collect()],
    )
}

fn component_table(rows: Vec<ComponentTableRow>, units: &UnitsSnapshot) -> String {
    let headers = [
        "Index".to_owned(),
        "Name".to_owned(),
        format!("IA / {}", units.amount),
        format!("A / {}", units.amount),
        "X".to_owned(),
        "AC".to_owned(),
        format!("MU / {}/{}", units.energy, units.amount),
        format!("Molar mass / {}/mol", units.amount),
        "Stoichiometry".to_owned(),
    ];
    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.index.to_string(),
                r.name,
                number(r.ia),
                number(r.a),
                number(r.x),
                number(r.ac),
                number(r.mu),
                number(r.wmass),
                vector(&r.stoic),
            ]
        })
        .collect();
    render("System components", &headers, rows)
}

fn phase_table(rows: Vec<PhaseTableRow>, units: &UnitsSnapshot) -> String {
    let headers = [
        "Index".to_owned(),
        "Name".to_owned(),
        "Model".to_owned(),
        "Status".to_owned(),
        "Stable".to_owned(),
        format!("A / {}", units.amount),
        "AC".to_owned(),
        format!("MU / {}/{}", units.energy, units.amount),
        format!("H / {}", units.energy),
        format!("S / {}/K", units.energy),
        format!("G / {}", units.energy),
        format!("Cp / {}/K", units.energy),
        format!("V / {}", units.volume),
        format!("HM / {}/{}", units.energy, units.amount),
        format!("SM / {}/({} K)", units.energy, units.amount),
        format!("GM / {}/{}", units.energy, units.amount),
        format!("CpM / {}/({} K)", units.energy, units.amount),
        format!("VM / {}/{}", units.volume, units.amount),
    ];
    let rows = rows
        .into_iter()
        .map(|r| {
            let mut row = vec![
                r.index.to_string(),
                r.name,
                r.model,
                r.status,
                r.stable.to_string(),
            ];
            row.extend(r.values.into_iter().map(number));
            row
        })
        .collect();
    render("Phases", &headers, rows)
}

fn constituent_table(rows: Vec<ConstituentTableRow>, units: &UnitsSnapshot) -> String {
    let headers = [
        "Phase".to_owned(),
        "Index".to_owned(),
        "Name".to_owned(),
        "Status".to_owned(),
        "Incoming".to_owned(),
        "Charge".to_owned(),
        format!("Molar mass / {}/mol", units.amount),
        format!("IA / {}", units.amount),
        format!("A / {}", units.amount),
        "AC".to_owned(),
        format!("MU / {}/{}", units.energy, units.amount),
        format!("H / {}", units.energy),
        format!("S / {}/K", units.energy),
        format!("G / {}", units.energy),
        format!("Cp / {}/K", units.energy),
        format!("V / {}", units.volume),
        format!("HM / {}/{}", units.energy, units.amount),
        format!("SM / {}/({} K)", units.energy, units.amount),
        format!("GM / {}/{}", units.energy, units.amount),
        format!("CpM / {}/({} K)", units.energy, units.amount),
        format!("VM / {}/{}", units.volume, units.amount),
        "Stoichiometry".to_owned(),
    ];
    let rows = rows
        .into_iter()
        .map(|r| {
            let mut row = vec![
                r.phase,
                r.index.to_string(),
                r.name,
                r.status,
                r.incoming.to_string(),
                number(r.charge),
                number(r.wmass),
            ];
            row.extend(r.values.into_iter().map(number));
            row.push(vector(&r.stoic));
            row
        })
        .collect();
    render("Phase constituents", &headers, rows)
}

fn species_table(rows: Vec<SpeciesTableRow>) -> String {
    let headers = ["Phase", "Sublattice", "Index", "Name", "X"].map(String::from);
    render(
        "Sublattice species",
        &headers,
        rows.into_iter()
            .map(|r| {
                vec![
                    r.phase,
                    r.sublattice.to_string(),
                    r.index.to_string(),
                    r.name,
                    number(r.x),
                ]
            })
            .collect(),
    )
}

fn bond_table(rows: Vec<BondTableRow>) -> String {
    let headers = ["Phase", "Model", "Kind", "Members", "X"].map(String::from);
    render(
        "TQBOND pairs and quadruplets",
        &headers,
        rows.into_iter()
            .map(|r| vec![r.phase, r.model, r.kind.to_owned(), r.members, number(r.x)])
            .collect(),
    )
}

pub(crate) fn snapshot_report(snapshot: &CalculatorSnapshot) -> String {
    let mut sections = vec![
        format!(
            "Snapshot filter: stable_only={}",
            snapshot.options().stable_only
        ),
        system_table(SystemTableRow::from(snapshot.system()), snapshot.units()),
        component_table(
            snapshot
                .components()
                .iter()
                .map(ComponentTableRow::from)
                .collect(),
            snapshot.units(),
        ),
        phase_table(
            snapshot.phases().iter().map(PhaseTableRow::from).collect(),
            snapshot.units(),
        ),
    ];
    let relation_headers = [
        "Phase".to_owned(),
        "Component".to_owned(),
        "XP".to_owned(),
        format!("AP / {}", snapshot.units().amount),
    ];
    let relation_rows = snapshot
        .phases()
        .iter()
        .flat_map(|phase| {
            phase.components.iter().map(move |component| {
                let component_name = snapshot
                    .components()
                    .iter()
                    .find(|candidate| candidate.index == component.component_index)
                    .map(|candidate| candidate.name.clone())
                    .unwrap_or_else(|| format!("component #{}", component.component_index));
                vec![
                    format!("{} [{}]", phase.name, phase.index),
                    format!("{} [{}]", component_name, component.component_index),
                    number(component.xp),
                    number(component.ap),
                ]
            })
        })
        .collect();
    sections.push(render(
        "Phase component composition",
        &relation_headers,
        relation_rows,
    ));
    sections.push(constituent_table(
        snapshot
            .phases()
            .iter()
            .flat_map(|p| p.constituents.iter())
            .map(ConstituentTableRow::from)
            .collect(),
        snapshot.units(),
    ));
    sections.push(species_table(
        snapshot
            .phases()
            .iter()
            .flat_map(|p| p.species.iter())
            .map(SpeciesTableRow::from)
            .collect(),
    ));
    sections.push(bond_table(
        snapshot
            .phases()
            .iter()
            .flat_map(|p| p.bonds.iter())
            .map(BondTableRow::from)
            .collect(),
    ));
    sections.join("\n\n")
}

pub(crate) fn stream_snapshot_table(snapshot: &StreamSnapshot) -> String {
    let row = StreamTableRow::from(snapshot);
    let u = &snapshot.units;
    let headers = [
        "Name".to_owned(),
        format!("T / {}", u.temperature),
        format!("P / {}", u.pressure),
        format!("Cp / {}/K", u.energy),
        format!("H / {}", u.energy),
        format!("S / {}/K", u.energy),
        format!("G / {}", u.energy),
        format!("V / {}", u.volume),
    ];
    let mut values = vec![row.name];
    values.extend(row.values.into_iter().map(number));
    render("Stream", &headers, vec![values])
}

pub(crate) fn live_report(calculator: &crate::Calculator) -> Result<String, ChemAppError> {
    let units = UnitsSnapshot::new(calculator)?;
    let system = system_table(SystemTableRow::try_from(&calculator.system())?, &units);
    let component_entities: Vec<_> = calculator.components()?.collect();
    let components = component_table(
        component_entities
            .iter()
            .map(ComponentTableRow::try_from)
            .collect::<Result<Vec<_>, _>>()?,
        &units,
    );
    let mut phase_rows = Vec::new();
    let mut relation_rows = Vec::new();
    let mut constituent_rows = Vec::new();
    let mut species_rows = Vec::new();
    let mut bond_rows = Vec::new();
    for phase in calculator.phases()? {
        phase_rows.push(PhaseTableRow::try_from(&phase)?);
        let phase_name = phase.name()?;
        for component in &component_entities {
            relation_rows.push(vec![
                format!("{} [{}]", phase_name, phase.index()),
                format!("{} [{}]", component.name()?, component.index()),
                number(component.xp(&phase)?),
                number(component.ap(&phase)?),
            ]);
        }
        for constituent in phase.constituents()? {
            constituent_rows.push(ConstituentTableRow::try_from(&constituent)?);
        }
        for species in phase.species()? {
            species_rows.push(SpeciesTableRow::try_from(&species)?);
        }
        for bond in phase.bonds()? {
            bond_rows.push(BondTableRow::try_from(&bond)?);
        }
    }
    let relation_headers = [
        "Phase".to_owned(),
        "Component".to_owned(),
        "XP".to_owned(),
        format!("AP / {}", units.amount),
    ];
    Ok(vec![
        "Snapshot filter: stable_only=false".to_owned(),
        system,
        components,
        phase_table(phase_rows, &units),
        render(
            "Phase component composition",
            &relation_headers,
            relation_rows,
        ),
        constituent_table(constituent_rows, &units),
        species_table(species_rows),
        bond_table(bond_rows),
    ]
    .join("\n\n"))
}

pub(crate) fn live_stream_table(stream: &Stream<'_>) -> Result<String, ChemAppError> {
    Ok(stream_snapshot_table(&stream.snapshot()?))
}

pub(crate) fn live_system_table(system: &System<'_>) -> Result<String, ChemAppError> {
    let units = UnitsSnapshot::new(system.calculator)?;
    Ok(system_table(SystemTableRow::try_from(system)?, &units))
}

pub(crate) fn live_component_table(
    component: &SystemComponent<'_>,
) -> Result<String, ChemAppError> {
    let units = UnitsSnapshot::new(component.calculator)?;
    Ok(component_table(
        vec![ComponentTableRow::try_from(component)?],
        &units,
    ))
}

pub(crate) fn live_phase_table(phase: &Phase<'_>) -> Result<String, ChemAppError> {
    let units = UnitsSnapshot::new(phase.calculator)?;
    Ok(phase_table(vec![PhaseTableRow::try_from(phase)?], &units))
}

pub(crate) fn live_constituent_table(
    constituent: &Constituent<'_>,
) -> Result<String, ChemAppError> {
    let units = UnitsSnapshot::new(constituent.calculator)?;
    Ok(constituent_table(
        vec![ConstituentTableRow::try_from(constituent)?],
        &units,
    ))
}

pub(crate) fn live_species_table(species: &Species<'_>) -> Result<String, ChemAppError> {
    Ok(species_table(vec![SpeciesTableRow::try_from(species)?]))
}

pub(crate) fn live_bond_table(bond: &Bond<'_>) -> Result<String, ChemAppError> {
    Ok(bond_table(vec![BondTableRow::try_from(bond)?]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::species::SpeciesRef;
    use crate::snapshot::{BondSnapshotKind, PairMemberSnapshot, QuadrupletMemberSnapshot};

    fn units() -> UnitsSnapshot {
        UnitsSnapshot {
            temperature: "K".into(),
            pressure: "bar".into(),
            volume: "dm3".into(),
            energy: "J".into(),
            amount: "mol".into(),
        }
    }

    #[test]
    fn numeric_and_vector_formatting_are_deterministic() {
        assert_eq!(number(1.25), "1.25000000e0");
        assert_eq!(vector(&[1.0, -2.0]), "[1.00000000e0, -2.00000000e0]");
    }

    #[test]
    fn pair_table_has_no_fake_third_or_fourth_member() {
        let snapshot = BondSnapshot {
            phase_index: 2,
            phase_name: "Liquid".into(),
            model: "QUAS".into(),
            kind: BondSnapshotKind::Pair {
                constituent_a: PairMemberSnapshot {
                    constituent_index: 1,
                    name: "A".into(),
                },
                constituent_b: PairMemberSnapshot {
                    constituent_index: 2,
                    name: "B".into(),
                },
            },
            x: 0.125,
        };
        let table = bond_table(vec![BondTableRow::from(&snapshot)]);
        assert!(table.contains("Pair"));
        assert!(table.contains("A [1] - B [2]"));
        assert!(!table.contains("species 3"));
    }

    #[test]
    fn every_entity_schema_renders_its_required_identity_columns() {
        let system = system_table(SystemTableRow { values: [1.0; 9] }, &units());
        assert!(system.contains("T / K") && system.contains("VT / dm3"));

        let components = component_table(
            vec![ComponentTableRow {
                index: 1,
                name: "Si".into(),
                ia: 1.0,
                a: 1.0,
                x: 1.0,
                ac: 1.0,
                mu: 0.0,
                wmass: 1.0,
                stoic: vec![1.0, 0.0],
            }],
            &units(),
        );
        assert!(components.contains("Stoichiometry") && components.contains("Si"));

        let phases = phase_table(
            vec![PhaseTableRow {
                index: 2,
                name: "Liquid".into(),
                model: "QUAS".into(),
                status: "ENTERED".into(),
                stable: true,
                values: [1.0; 13],
            }],
            &units(),
        );
        assert!(phases.contains("Stable") && phases.contains("QUAS"));

        let constituents = constituent_table(
            vec![ConstituentTableRow {
                phase: "Liquid [2]".into(),
                index: 1,
                name: "A".into(),
                status: "ENTERED".into(),
                incoming: true,
                charge: 0.0,
                wmass: 1.0,
                values: [1.0; 14],
                stoic: vec![1.0],
            }],
            &units(),
        );
        assert!(constituents.contains("Incoming") && constituents.contains("Liquid [2]"));

        let species = species_table(vec![SpeciesTableRow {
            phase: "Slag [3]".into(),
            sublattice: 2,
            index: 4,
            name: "O".into(),
            x: 0.25,
        }]);
        assert!(species.contains("Sublattice") && species.contains("Slag [3]"));

        let stream = StreamSnapshot {
            units: units(),
            name: "FEED".into(),
            temperature: 298.15,
            pressure: 1.0,
            cp: 1.0,
            h: 2.0,
            s: 3.0,
            g: 4.0,
            v: 5.0,
        };
        let stream_table = stream_snapshot_table(&stream);
        assert!(stream_table.contains("Name") && stream_table.contains("FEED"));
    }

    #[test]
    fn pair_and_quadruplet_rows_are_distinguishable() {
        let pair = BondSnapshot {
            phase_index: 1,
            phase_name: "Pair phase".into(),
            model: "QSOL".into(),
            kind: BondSnapshotKind::Pair {
                constituent_a: PairMemberSnapshot {
                    constituent_index: 1,
                    name: "A".into(),
                },
                constituent_b: PairMemberSnapshot {
                    constituent_index: 1,
                    name: "A".into(),
                },
            },
            x: 0.5,
        };
        let member = |sublattice, local_index, name: &str| QuadrupletMemberSnapshot {
            identity: SpeciesRef {
                sublattice,
                local_index,
            },
            name: name.into(),
        };
        let quadruplet = BondSnapshot {
            phase_index: 2,
            phase_name: "Slag".into(),
            model: "SUBG".into(),
            kind: BondSnapshotKind::Quadruplet {
                species_a: member(1, 1, "Ca"),
                species_b: member(1, 2, "Mg"),
                species_c: member(2, 1, "O"),
                species_d: member(2, 2, "F"),
            },
            x: 0.25,
        };
        let table = bond_table(vec![
            BondTableRow::from(&pair),
            BondTableRow::from(&quadruplet),
        ]);
        assert!(table.contains("Pair") && table.contains("Quadruplet"));
        assert_eq!(table.matches("Pair phase [1]").count(), 1);
        assert_eq!(table.matches("Slag [2]").count(), 1);
        assert!(table.contains("Ca [S1:1], Mg [S1:2] | O [S2:1], F [S2:2]"));
    }
}
