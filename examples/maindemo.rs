// chemapp_rs//examples/maindemo.rs
mod common;

use chemapp_rs::{ChemAppError, Engine};

pub fn main() -> Result<(), ChemAppError> {
    /**********************************************************************************************************************/
    let libpath = common::required_path("CHEMAPP_LIBRARY", "ChemApp DLL or shared library")?;
    /**********************************************************************************************************************/
    let datafile_path =
        common::required_path("CHEMAPP_DATAFILE", "C-O-Si cademo-compatible DAT file")?;
    let libname = common::path_text(&libpath, "ChemApp library")?;
    let datafile_dat = common::path_text(&datafile_path, "thermodynamic data-file")?;
    /**********************************************************************************************************************/
    let engine = Engine::new(libname)?;
    /**********************************************************************************************************************/
    // Initialize the library
    engine.tqini().unwrap();
    /**********************************************************************************************************************/
    // TQERR reads ChemApp's current message buffer, so retrieve the copyright
    // records immediately after TQCPRT, as the GTT C demo does.
    engine.tqcprt().unwrap();
    let cprt = engine.tqerr().unwrap();
    println!("ChemApp copyright message:\n{}", cprt);
    /**********************************************************************************************************************/
    // ChemApp library version
    let vers = engine.tqvers().unwrap();
    println!("ChemApp version = {:?}", vers);
    /**********************************************************************************************************************/
    // Internal array sizes
    let sizes = engine.tqsize().unwrap();
    println!("Internal array sizes:\n{:?}", sizes);
    /**********************************************************************************************************************/
    // default FORTRAN unit for tqrfil
    let unitno = engine.tqgio("FILE").unwrap();
    println!(
        "The thermochemical data will be read from the file associated with unit {:?}",
        unitno
    );
    /**********************************************************************************************************************/
    engine.tqopna(datafile_dat, unitno).unwrap();
    engine.tqrfil().unwrap();
    engine.tqclos(unitno).unwrap();
    /**********************************************************************************************************************/
    // Used array dimensions
    let used = engine.tqused().unwrap();
    println!("Used array sizes:\n{:?}", used);
    /**********************************************************************************************************************/
    /**********************************************************************************************************************/
    // get system units
    let punit = engine.tqgsu("Pressure").unwrap();
    let vunit = engine.tqgsu("Volume").unwrap();
    let tunit = engine.tqgsu("Temperature").unwrap();
    let eunit = engine.tqgsu("Energy").unwrap();
    let aunit = engine.tqgsu("Amount").unwrap();
    println!("Pressure unit: {:?}", punit);
    println!("Volume unit: {:?}", vunit);
    println!("Temperature unit: {:?}", tunit);
    println!("Energy unit: {:?}", eunit);
    println!("Amount unit: {:?}", aunit);
    /**********************************************************************************************************************/
    // change "Amount" unit to grams
    engine.tqcsu("Amount", "gram").unwrap();
    /**********************************************************************************************************************/
    /**********************************************************************************************************************/
    let nscom = engine.tqnosc().unwrap();
    println!("Number of system components : {:?}", nscom);
    /**********************************************************************************************************************/
    let name = engine.tqgnsc(1).unwrap();
    let (stoic, wmass) = engine.tqstsc(1).unwrap();
    println!(
        "System component {:?}, stoic = {:?}, wmass = {:?}",
        name, stoic, wmass
    );
    let index = engine.tqinsc(&name).unwrap();
    println!("Index number of {:?} is {:?}", name, index);
    /**********************************************************************************************************************/
    let newsyscomp = vec!["SiO", "SiC", "CO"];
    engine.tqcsc(&newsyscomp).unwrap();
    println!("System components changed to {:?}", newsyscomp);
    for k in 1..=nscom {
        let name = engine.tqgnsc(k).unwrap();
        let (stoic, wmass) = engine.tqstsc(k).unwrap();
        println!(
            "Name of new system component {:?}: {:?}, stoic = {:?}, wmass = {:?}",
            k, name, stoic, wmass
        );
    }
    /**********************************************************************************************************************/
    /**********************************************************************************************************************/
    // Continue cademo1.c immediately after its first tqcsc section.
    let (stoic, wmass) = engine.tqstsc(1).unwrap();
    println!(
        "Updated component 1: stoic = {:?}, wmass = {:?}",
        stoic, wmass
    );
    let nphase = engine.tqnop().unwrap();
    let phase_name = engine.tqgnp(1).unwrap();
    println!(
        "Phases: {:?}; phase 1: {:?}; reverse lookup: {:?}",
        nphase,
        phase_name,
        engine.tqinp(&phase_name).unwrap()
    );
    println!(
        "Models: phase 1 = {:?}, last phase = {:?}",
        engine.tqmodl(1).unwrap(),
        engine.tqmodl(nphase).unwrap()
    );
    let npcgas = engine.tqnopc(1).unwrap();
    let constituent_name = engine.tqgnpc(1, 1).unwrap();
    println!(
        "Phase 1 constituents: {:?}; first: {:?}; reverse lookup: {:?}",
        npcgas,
        constituent_name,
        engine.tqinpc(1, &constituent_name).unwrap()
    );
    println!(
        "Incoming species permitted: {:?}",
        engine.tqpcis(1, 1).unwrap()
    );
    let (stoic, wmass) = engine.tqstpc(1, 1).unwrap();
    println!("Phase constituent stoic = {:?}, wmass = {:?}", stoic, wmass);

    engine.tqcsc(&["C", "O", "Si"]).unwrap();
    engine.tqcsp(1, "eliminated").unwrap();
    println!("Phase status: {:?}", engine.tqgsp(1).unwrap());
    engine.tqcsp(1, "entered").unwrap();
    println!("Phase status: {:?}", engine.tqgsp(1).unwrap());
    engine.tqcspc(1, 1, "dormant").unwrap();
    println!("Constituent status: {:?}", engine.tqgspc(1, 1).unwrap());
    engine.tqcspc(1, 1, "entered").unwrap();
    println!("Constituent status: {:?}", engine.tqgspc(1, 1).unwrap());

    let _ = engine.tqsetc("ia ", 1, 4, 1.0).unwrap();
    engine.tqcsu("Amount", "mol").unwrap();
    let _ = engine.tqsetc("ia ", 1, 12, 3.0).unwrap();
    let numcon = engine.tqsetc("ia ", 1, 8, 2.0).unwrap();
    engine.tqremc(numcon).unwrap();
    let _ = engine.tqsetc("t ", 0, 0, 1800.0).unwrap();
    engine.tqclim("plow", 1e-49).unwrap();
    engine.tqshow().unwrap();
    engine.tqce(" ", 0, 0, (0.0, 0.0)).unwrap();
    engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
    let _ = engine.tqsetc("t ", 0, 0, 1850.0).unwrap();
    engine.tqcen(" ", 0, 0, (0.0, 0.0)).unwrap();
    let _ = engine.tqsetc("t ", 0, 0, 1900.0).unwrap();
    engine.tqcenl(" ", 0, 0, (0.0, 0.0)).unwrap();

    let result_path = std::env::temp_dir().join("chemapp_rs_maindemo_result.txt");
    engine.tqcio("LIST", 21).unwrap();
    engine.tqopen(result_path.to_str().unwrap(), 21).unwrap();
    engine
        .tqwstr("LIST", "Output from tqcel (ChemSage result table):")
        .unwrap();
    engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
    engine.tqclos(21).unwrap();
    engine.tqcio("LIST", 6).unwrap();

    let component_name = engine.tqgnsc(1).unwrap();
    println!(
        "Mole fraction of {:?}: {:?}",
        component_name,
        engine.tqgetr("xp ", 1, 1).unwrap()
    );
    println!(
        "Equilibrium amount: {:?}; phase activity: {:?}",
        engine.tqgetr("a ", 1, 1).unwrap(),
        engine.tqgetr("ac ", 1, 0).unwrap()
    );
    for indexc in 1..=npcgas {
        println!(
            "Fugacity of {:?}: {:?}",
            engine.tqgnpc(1, indexc).unwrap(),
            engine.tqgetr("ac", 1, indexc).unwrap()
        );
    }
    println!("Dimensionless G: {:?}", engine.tqgdpc("G", 1, 1).unwrap());

    let is_light = engine.tqlite().unwrap();
    if is_light {
        println!("Target calculations omitted for the ChemApp light version.");
    } else {
        let liquid = engine.tqinp("SiO2(liq").unwrap();
        let _ = engine.tqsetc("a", liquid, 0, 0.0).unwrap();
        engine.tqcel("t", 0, 0, (2000.0, 0.0)).unwrap();
        println!(
            "Formation temperature: {:?}",
            engine.tqgetr("t", 0, 0).unwrap()
        );
        engine.tqremc(-2).unwrap();
        let quartz = engine.tqinp("SiO2(quartz)").unwrap();
        let _ = engine.tqsetc("IA", quartz, 0, 1.0).unwrap();
        let interval = (300.0, 3000.0);
        let mut more = engine.tqmap("tf", 0, 0, interval).unwrap();
        let mut result_number = 1;
        println!("Mapping result: {:?} K", engine.tqgetr("t", 0, 0).unwrap());
        while more > 0 {
            more = if result_number == 2 {
                engine.tqmapl("tn", 0, 0, interval).unwrap()
            } else {
                engine.tqmap("tn", 0, 0, interval).unwrap()
            };
            result_number += 1;
            println!("Mapping result: {:?} K", engine.tqgetr("t", 0, 0).unwrap());
        }
    }

    engine.tqremc(-2).unwrap();
    for stream in ["stream1", "stream2", "stream3"] {
        engine.tqsttp(stream, (1000.0, 1.0)).unwrap();
    }
    engine.tqstca("stream1", 1, 4, 1.0).unwrap();
    engine.tqstca("stream2", 1, 12, 3.0).unwrap();
    engine.tqstca("stream3", 1, 8, 2.0).unwrap();
    engine.tqstrm("stream3").unwrap();
    engine.tqstec("t ", 0, 1800.0).unwrap();
    engine.tqcel(" ", 0, 0, (0.0, 0.0)).unwrap();
    println!(
        "Enthalpy of stream1: {:?}",
        engine.tqstxp("stream1", "H").unwrap()
    );

    if let Some(sublattice_file) =
        std::env::var_os("CHEMAPP_SUBLATTICE_DATAFILE").map(std::path::PathBuf::from)
    {
        engine
            .tqopna(sublattice_file.to_str().unwrap(), unitno)
            .unwrap();
        engine.tqrfil().unwrap();
        engine.tqclos(unitno).unwrap();
        let sigma = engine.tqinp("SIGMA:30#1").unwrap();
        for indexl in 1..=engine.tqnosl(sigma).unwrap() {
            for indexc in 1..=engine.tqnolc(sigma, indexl).unwrap() {
                let name = engine.tqgnlc(sigma, indexl, indexc).unwrap();
                println!(
                    "Sublattice {:?}, constituent {:?}: {:?}",
                    indexl,
                    engine.tqinlc(&name, sigma, indexl).unwrap(),
                    name
                );
            }
        }
        let _ = engine.tqsetc("T", 0, 0, 1000.0).unwrap();
        for (name, amount) in [("Co", 0.25), ("Cr", 0.25), ("Fe", 0.50)] {
            let indexc = engine.tqinsc(name).unwrap();
            let _ = engine.tqsetc("ia", 0, indexc, amount).unwrap();
        }
        engine.tqce(" ", 0, 0, (0.0, 0.0)).unwrap();
        for indexp in 1..=engine.tqnop().unwrap() {
            let model = engine.tqmodl(indexp).unwrap();
            if engine.tqgetr("a", indexp, 0).unwrap() > 0.0
                && !model.trim().eq_ignore_ascii_case("PURE")
            {
                println!(
                    "Sublattice fractions in {:?}",
                    engine.tqgnp(indexp).unwrap()
                );
                for indexl in 1..=engine.tqnosl(indexp).unwrap() {
                    for indexc in 1..=engine.tqnolc(indexp, indexl).unwrap() {
                        println!(
                            "{:?}: {:?}",
                            engine.tqgnlc(indexp, indexl, indexc).unwrap(),
                            engine.tqgtlc(indexp, indexl, indexc).unwrap()
                        );
                    }
                }
            }
        }
    } else {
        println!(
            "Skipping sublattice functions; set CHEMAPP_SUBLATTICE_DATAFILE to exercise them."
        );
    }

    // Exercise license/interface getters without printing installation-specific
    // identifiers, holder names, or dongle values in ordinary demo logs.
    let license_id = engine.tqgtid().unwrap();
    let license_holder = engine.tqgtnm().unwrap();
    let program_id = engine.tqgtpi().unwrap();
    let (dongle_type, _dongle_id) = engine.tqgthi().unwrap();
    let (expiry_month, expiry_year) = engine.tqgted().unwrap();
    println!("ChemApp Light mode: {:?}", is_light);
    println!("License ID returned: {}", !license_id.is_empty());
    println!("License holder returned: {}", !license_holder.is_empty());
    println!(
        "License holder contains internal spaces: {}",
        license_holder.contains(' ')
    );
    println!("Program ID returned: {}", !program_id.is_empty());
    println!("Dongle mechanism returned: {}", !dongle_type.is_empty());
    println!(
        "Expiration information returned: {}",
        expiry_month > 0 || expiry_year > 0
    );
    let error_unit = engine.tqgio("ERROR").unwrap();
    engine.tqcio("ERROR", 0).unwrap();
    let transparent_file =
        std::env::var_os("CHEMAPP_TRANSPARENT_DATAFILE").map(std::path::PathBuf::from);
    let transparent_opened = transparent_file.as_ref().is_some_and(|path| {
        path.to_str()
            .is_some_and(|path| engine.tqopnt(path, unitno).is_ok())
    });
    engine.tqcio("ERROR", error_unit).unwrap();
    if transparent_opened {
        engine.tqrcst().unwrap();
        engine.tqclos(unitno).unwrap();
        println!("Transparent file header: {:#?}", engine.tqgtrh().unwrap());
    } else {
        println!("Skipping transparent file functions; set CHEMAPP_TRANSPARENT_DATAFILE to exercise them.");
    }
    println!("End of output translated from cademo1.");
    Ok(())
}
